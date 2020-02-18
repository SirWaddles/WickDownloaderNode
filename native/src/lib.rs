use neon::prelude::*;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::runtime;
use wickdl::{ServiceState, EncryptedPak, PakService};

// https://stackoverflow.com/a/27650405/3479580
struct HexSlice<'a>(&'a [u8]);

impl<'a> HexSlice<'a> {
    fn new<T>(data: &'a T) -> HexSlice<'a> 
        where T: ?Sized + AsRef<[u8]> + 'a
    {
        HexSlice(data.as_ref())
    }
}

impl<'a> fmt::Display for HexSlice<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{:02X}", byte)?;
        }
        Ok(())
    }
}

pub struct RuntimeContainerInner {
    runtime: runtime::Runtime,
    // The mutex is only there to set the Option. Please kill me.
    service: Mutex<Option<Arc<ServiceState>>>,
    next_counter: AtomicU32,
}

pub struct RuntimeContainer {
    inner: Option<Arc<RuntimeContainerInner>>,
    cb: Option<EventHandler>,
}

pub struct EncryptedPakContainer {
    pak: Option<EncryptedPak>,
}

pub struct PakContainer {
    pak: Option<Arc<PakService>>,
}

declare_types! {
    pub class JsRuntimeContainer for RuntimeContainer {
        init(mut cx) {
            let rt = runtime::Builder::new()
                .enable_all()
                .threaded_scheduler()
                .build().unwrap();

            let js_cb = cx.argument::<JsFunction>(0)?;
            let this = cx.this();
            let cb = EventHandler::new(&cx, this, js_cb);

            Ok(RuntimeContainer {
                inner: Some(Arc::new(RuntimeContainerInner {
                    runtime: rt,
                    service: Mutex::new(None),
                    next_counter: AtomicU32::new(1),
                })),
                cb: Some(cb),
            })
        }

        method start_service(mut cx) {
            let this = cx.this();
            let (cb, state) = {
                let guard = cx.lock();
                let data = this.borrow(&guard);
                (data.cb.as_ref().unwrap().clone(), Arc::clone(&data.inner.as_ref().unwrap()))
            };
            let counter = state.next_counter.fetch_add(1, Ordering::SeqCst) as f64;

            // Still don't know how to do this properly
            let state2 = Arc::clone(&state);

            state.runtime.spawn(async move {
                match ServiceState::new().await {
                    Ok(service) => {
                        let mut lock = state2.service.lock().unwrap();
                        *lock = Some(Arc::new(service));
                        cb.schedule(move |tcx| -> Vec<Handle<JsValue>> {
                            vec![tcx.number(counter).upcast(), tcx.number(0.0).upcast()]
                        });
                    },
                    Err(err) => {
                        cb.schedule(move |tcx| -> Vec<Handle<JsValue>> {
                            let err = JsString::new(tcx, format!("Error: {}", err)).upcast();
                            vec![tcx.number(counter).upcast(), tcx.number(1.0).upcast(), err]
                        });
                    },
                }
            });

            Ok(cx.number(counter).upcast())
        }

        method start_with_manifest(mut cx) {
            let this = cx.this();
            let state = {
                let guard = cx.lock();
                let data = this.borrow(&guard);
                Arc::clone(&data.inner.as_ref().unwrap())
            };

            let app_manifest = cx.argument::<JsString>(0)?.value();
            let chunk_manifest = cx.argument::<JsString>(1)?.value();
            let service = match ServiceState::from_manifests(&app_manifest, &chunk_manifest) {
                Ok(d) => d,
                Err(_) => return cx.throw_error("Cannot parse manifests"),
            };

            {
                let mut lock = state.service.lock().unwrap();
                *lock = Some(Arc::new(service));
            }

            Ok(cx.undefined().upcast())
        }

        method get_paks(mut cx) {
            let this = cx.this();
            let state = {
                let guard = cx.lock();
                let data = this.borrow(&guard);
                Arc::clone(&data.inner.as_ref().unwrap())
            };

            let service = Arc::clone(state.service.lock().unwrap().as_ref().unwrap());
            let names = service.get_paks();

            let js_array = JsArray::new(&mut cx, names.len() as u32);
            for (i, item) in names.iter().enumerate() {
                let js_string = cx.string(item);
                js_array.set(&mut cx, i as u32, js_string)?;
            }

            Ok(js_array.upcast())
        }

        method download_pak(mut cx) {
            let this = cx.this();
            let pak_name = cx.argument::<JsString>(0)?.value();
            let target_file = cx.argument::<JsString>(1)?.value();
            let (cb, state) = {
                let guard = cx.lock();
                let data = this.borrow(&guard);
                (data.cb.as_ref().unwrap().clone(), Arc::clone(&data.inner.as_ref().unwrap()))
            };

            let service = Arc::clone(state.service.lock().unwrap().as_ref().unwrap());
            let counter = state.next_counter.fetch_add(1, Ordering::SeqCst) as f64;

            state.runtime.spawn(async move {
                match service.download_pak(pak_name, target_file).await {
                    Ok(_) => {
                        cb.schedule(move |tcx| -> Vec<Handle<JsValue>> {
                            vec![tcx.number(counter).upcast(), tcx.number(0.0).upcast()]
                        });
                    },
                    Err(err) => {
                        cb.schedule(move |tcx| -> Vec<Handle<JsValue>> {
                            let err = JsString::new(tcx, format!("Error: {}", err)).upcast();
                            vec![tcx.number(counter).upcast(), tcx.number(1.0).upcast(), err]
                        });
                    },
                }
            });

            Ok(cx.number(counter).upcast())
        }

        method get_pak(mut cx) {
            let this = cx.this();
            let pak_name = cx.argument::<JsString>(0)?.value();
            let (cb, state) = {
                let guard = cx.lock();
                let data = this.borrow(&guard);
                (data.cb.as_ref().unwrap().clone(), Arc::clone(&data.inner.as_ref().unwrap()))
            };

            let service = Arc::clone(state.service.lock().unwrap().as_ref().unwrap());
            let counter = state.next_counter.fetch_add(1, Ordering::SeqCst) as f64;

            state.runtime.spawn(async move {
                match service.get_pak(pak_name).await {
                    Ok(pak) => {
                        cb.schedule(move |tcx| -> Vec<Handle<JsValue>> {
                            let mut pak_container = JsEncryptedPak::new::<_, JsEncryptedPak, _>(tcx, vec![]).unwrap();
                            let guard = tcx.lock();
                            pak_container.borrow_mut(&guard).pak = Some(pak);
                            vec![tcx.number(counter).upcast(), tcx.number(0.0).upcast(), pak_container.upcast()]
                        });
                    },
                    Err(err) => {
                        cb.schedule(move |tcx| -> Vec<Handle<JsValue>> {
                            let err = JsString::new(tcx, format!("Error: {}", err)).upcast();
                            vec![tcx.number(counter).upcast(), tcx.number(1.0).upcast(), err]
                        });
                    },
                }
            });

            Ok(cx.number(counter).upcast())
        }

        method decrypt_pak(mut cx) {
            let this = cx.this();
            let mut pak = cx.argument::<JsEncryptedPak>(0)?;
            let key = cx.argument::<JsString>(1)?.value();
            let (cb, state) = {
                let guard = cx.lock();
                let data = this.borrow(&guard);
                (data.cb.as_ref().unwrap().clone(), Arc::clone(&data.inner.as_ref().unwrap()))
            };
            let encpakop = {
                let guard = cx.lock();
                let mut data = pak.borrow_mut(&guard);
                data.pak.take()
            };
            let encpak = match encpakop {
                Some(inner) => inner,
                None => return cx.throw_error("Pak already consumed"),
            };

            let service = Arc::clone(state.service.lock().unwrap().as_ref().unwrap());
            let counter = state.next_counter.fetch_add(1, Ordering::SeqCst) as f64;

            state.runtime.spawn(async move {
                match service.decrypt_pak(encpak, key).await {
                    Ok(pak) => {
                        cb.schedule(move |tcx| -> Vec<Handle<JsValue>> {
                            let mut pak_container = JsPakContainer::new::<_, JsPakContainer, _>(tcx, vec![]).unwrap();
                            let guard = tcx.lock();
                            pak_container.borrow_mut(&guard).pak = Some(Arc::new(pak));
                            vec![tcx.number(counter).upcast(), tcx.number(0.0).upcast(), pak_container.upcast()]
                        })
                    },
                    Err(err) => {
                        cb.schedule(move |tcx| -> Vec<Handle<JsValue>> {
                            let err = JsString::new(tcx, format!("Error: {}", err)).upcast();
                            vec![tcx.number(counter).upcast(), tcx.number(1.0).upcast(), err]
                        })
                    },
                }
            });

            Ok(cx.number(counter).upcast())
        }

        method get_file_data(mut cx) {
            let this = cx.this();
            let pak = cx.argument::<JsPakContainer>(0)?;
            let file = cx.argument::<JsString>(1)?.value();
            let (cb, state) = {
                let guard = cx.lock();
                let data = this.borrow(&guard);
                (data.cb.as_ref().unwrap().clone(), Arc::clone(&data.inner.as_ref().unwrap()))
            };
            let pak = {
                let guard = cx.lock();
                let data = pak.borrow(&guard);
                match &data.pak {
                    Some(inner) => Some(Arc::clone(&inner)),
                    None => None,
                }
            };
            let pak = match pak {
                Some(inner) => inner,
                None => return cx.throw_error("No pak inside container"),
            };
            let counter = state.next_counter.fetch_add(1, Ordering::SeqCst) as f64;

            state.runtime.spawn(async move {
                match pak.get_data(&file).await {
                    Ok(data) => {
                        cb.schedule(move |tcx| -> Vec<Handle<JsValue>> {
                            let buffer = {
                                let buffer = JsBuffer::new(tcx, data.len() as u32).unwrap();
                                let guard = tcx.lock();
                                let contents = buffer.borrow(&guard);
                                let slice = contents.as_mut_slice();
                                slice.copy_from_slice(&data);
                                buffer
                            };
                            vec![tcx.number(counter).upcast(), tcx.number(0.0).upcast(), buffer.upcast()]
                        });
                    },
                    Err(err) => {
                        cb.schedule(move |tcx| -> Vec<Handle<JsValue>> {
                            let err = JsString::new(tcx, format!("Error: {}", err)).upcast();
                            vec![tcx.number(counter).upcast(), tcx.number(1.0).upcast(), err]
                        });
                    },
                }
            });

            Ok(cx.number(counter).upcast())
        }

        method shutdown(mut cx) {
            let mut this = cx.this();
            {
                let guard = cx.lock();
                let mut data = this.borrow_mut(&guard);
                data.inner = None;
                data.cb = None;
            }
            Ok(cx.undefined().upcast())
        }
    }

    pub class JsEncryptedPak for EncryptedPakContainer {
        init(_) {
            Ok(EncryptedPakContainer {
                pak: None,
            })
        }
    }

    pub class JsPakContainer for PakContainer {
        init(_) {
            Ok(PakContainer {
                pak: None,
            })
        }

        method get_pak_mount(mut cx) {
            let this = cx.this();
            let contents = {
                let guard = cx.lock();
                let data = this.borrow(&guard);
                match &data.pak {
                    Some(inner) => Some(inner.get_mount_point().to_owned()),
                    None => None,
                }
            };
            let mount = match contents {
                Some(inner) => inner,
                None => return cx.throw_error("No pak inside container"),
            };

            Ok(JsString::new(&mut cx, mount).upcast())
        }

        method get_file_names(mut cx) {
            let this = cx.this();
            let contents = {
                let guard = cx.lock();
                let data = this.borrow(&guard);
                match &data.pak {
                    Some(inner) => Some(inner.get_files()),
                    None => None,
                }
            };
            let files = match contents {
                Some(inner) => inner,
                None => return cx.throw_error("No pak inside container"),
            };
            let ret_arr = JsArray::new(&mut cx, files.len() as u32);
            for (i, obj) in files.iter().enumerate() {
                let entry = cx.string(obj);
                ret_arr.set(&mut cx, i as u32, entry).unwrap();
            }
            Ok(ret_arr.upcast())
        }

        method get_file_hash(mut cx) {
            let this = cx.this();
            let filepath = cx.argument::<JsString>(0)?.value();

            let contents = {
                let guard = cx.lock();
                let data = this.borrow(&guard);
                match &data.pak {
                    Some(inner) => Some(inner.get_hash(&filepath)),
                    None => None,
                }
            };
            let hash = match contents {
                Some(inner) => match inner {
                    Ok(hash) => format!("{}", HexSlice::new(&hash)),
                    Err(_) => return cx.throw_error("File does not exist"),
                },
                None => return cx.throw_error("No pak inside container"),
            };

            Ok(JsString::new(&mut cx, hash).upcast())
        }
    }
}

register_module!(mut cx, {
    cx.export_class::<JsRuntimeContainer>("RuntimeContainer")?;

    Ok(())
});
