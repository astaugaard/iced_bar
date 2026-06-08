use std::{
    cell::RefCell,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Poll, Waker},
};

use smol::{
    channel::{self, Receiver},
    future::FutureExt,
};

use libpulse_binding::{
    context::{
        self, Context,
        subscribe::{Facility, InterestMaskSet, Operation},
    },
    mainloop::threaded::{self, Mainloop},
};

use crate::model::Message;

// enum AwaitCallback<'a> {
//     MustRegister(&'a mut Mainloop, &'a mut dyn FnMut(Box<dyn FnMut()>)),
//     Registered,
// }

// impl<'a> Future for AwaitCallback<'a> {
//     type Output = ();

//     fn poll(
//         self: std::pin::Pin<&mut Self>,
//         cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Self::Output> {
//         let inner = Pin::into_inner(self);

//         let res = match inner {
//             AwaitCallback::MustRegister(mainloop, f) => {
//                 let waker = cx.waker().clone();

//                 mainloop.lock();

//                 f(Box::new(move || waker.wake_by_ref()));

//                 mainloop.unlock();

//                 Poll::Pending
//             }
//             AwaitCallback::Registered => Poll::Ready(()),
//         };

//         *inner = Self::Registered;

//         res
//     }
// }

// struct AwaitCallbackResponce<'a, A> {
//     mainloop: &'a mut Mainloop,
//     to_register: &'a mut dyn FnMut(Box<dyn FnMut(A)>),
//     registered: bool,
//     answer: Rc<RefCell<Option<A>>>,
// }

// impl<'a, A> Future for AwaitCallbackResponce<'a, A>
// where
//     A: Unpin + 'static,
// {
//     type Output = A;

//     fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
//         let inner = Pin::into_inner(self);

//         if !inner.registered {
//             let waker = cx.waker().clone();

//             inner.mainloop.lock();

//             let ans = inner.answer.clone();

//             (inner.to_register)(Box::new(move |answer| {
//                 *ans.borrow_mut() = Some(answer);

//                 waker.wake_by_ref()
//             }));

//             inner.mainloop.unlock();

//             inner.registered = true;

//             Poll::Pending
//         } else {
//             Poll::Ready(inner.answer.replace(None).unwrap())
//         }
//     }
// }

struct AwaitCallbackUntil<'a, A> {
    mainloop: &'a mut Mainloop,
    to_register: &'a mut dyn FnMut(Box<dyn FnMut()>),
    registered: bool,
    until: &'a mut dyn FnMut() -> Option<A>,
    waker: Arc<Mutex<Option<Waker>>>,
}

impl<'a, A> Future for AwaitCallbackUntil<'a, A>
where
    A: Unpin + 'static,
{
    type Output = A;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let inner = Pin::into_inner(self);

        inner.mainloop.lock();

        if let Some(a) = (inner.until)() {
            inner.mainloop.unlock();
            return Poll::Ready(a);
        };

        *inner.waker.lock().unwrap() = Some(cx.waker().clone());

        if !inner.registered {
            let waker = inner.waker.clone();

            (inner.to_register)(Box::new(move || {
                waker.lock().unwrap().as_ref().unwrap().wake_by_ref()
            }));

            inner.registered = true;
        }

        inner.mainloop.unlock();

        Poll::Pending
    }
}

// async fn await_callback_pulseaudio<F>(mainloop: &mut Mainloop, mut f: F)
// where
//     F: FnMut(Box<dyn FnMut()>),
// {
//     AwaitCallback::MustRegister(mainloop, &mut f).await
// }

// async fn await_callback_pulseaudio_arg<F, A>(mainloop: &mut Mainloop, mut f: F) -> A
// where
//     F: FnMut(Box<dyn FnMut(A)>),
//     A: Unpin + 'static,
// {
//     AwaitCallbackResponce {
//         mainloop,
//         to_register: &mut f,
//         registered: false,
//         answer: Rc::new(RefCell::new(None)),
//     }
//     .await
// }

async fn await_callback_pulseaudio_until<F, G, A>(
    mainloop: &mut Mainloop,
    mut f: F,
    mut until: G,
) -> A
where
    F: FnMut(Box<dyn FnMut()>),
    G: FnMut() -> Option<A>,
    A: Unpin + 'static,
{
    AwaitCallbackUntil {
        mainloop,
        to_register: &mut f,
        registered: false,
        until: &mut until,
        waker: Arc::new(Mutex::new(None)),
    }
    .await
}

pub enum AudioCommands {
    Quit,
    Refresh(u32),
}

pub async fn init(
    information_channel: iced::futures::channel::mpsc::Sender<Message>,
    commands: Receiver<AudioCommands>,
) {
    let mut mainloop = threaded::Mainloop::new().unwrap();

    let context = RefCell::new(Context::new(&mainloop, "pulse audio volume listener").unwrap());

    context
        .borrow_mut()
        .connect(None, context::FlagSet::NOAUTOSPAWN, None)
        .unwrap();

    mainloop.lock();

    mainloop.start().unwrap();

    mainloop.unlock();

    let result = await_callback_pulseaudio_until(
        &mut mainloop,
        |wake| context.borrow_mut().set_state_callback(Some(wake)),
        || match context.borrow().get_state() {
            context::State::Unconnected => Some(false),
            context::State::Connecting => None,
            context::State::Authorizing => None,
            context::State::SettingName => None,
            context::State::Ready => Some(true),
            context::State::Failed => Some(false),
            context::State::Terminated => Some(false),
        },
    )
    .await;

    assert!(result);

    mainloop.lock();

    let (sender, subscription_responce) = channel::bounded(8);

    sender.send(AudioCommands::Refresh(0)).await.unwrap();

    context
        .borrow_mut()
        .set_subscribe_callback(Some(Box::new(move |facility, operation, idx| {
            if let (Some(Facility::Sink), Some(Operation::Changed)) = (facility, operation) {
                match sender.try_send(AudioCommands::Refresh(idx)) {
                    Ok(()) => {}
                    Err(err) => {
                        eprintln!("buffer full: {}", err)
                    }
                }
            }
        })));

    context
        .borrow_mut()
        .subscribe(InterestMaskSet::SINK, |success| assert!(success));
    mainloop.unlock();

    loop {
        match subscription_responce.recv().or(commands.recv()).await {
            Ok(AudioCommands::Quit) => break,
            Ok(AudioCommands::Refresh(idx)) => {
                let mut information_channel = information_channel.clone();

                mainloop.lock();

                context.borrow_mut().introspect().get_sink_info_by_index(
                    idx,
                    move |item| match item {
                        libpulse_binding::callbacks::ListResult::Item(a) => {
                            match information_channel.try_send(Message::NewVolume(a.volume.avg())) {
                                Ok(()) => {}
                                Err(err) => {
                                    eprintln!("send error on volume update {err}")
                                }
                            }
                        }
                        libpulse_binding::callbacks::ListResult::End => {}
                        libpulse_binding::callbacks::ListResult::Error => {
                            eprintln!("error getting sink info")
                        }
                    },
                );

                mainloop.unlock();
            }
            Err(err) => {
                eprintln!("recieve error: {}", err)
            }
        }
    }

    mainloop.stop();
}
