pub mod error;

use std::{io::Read, os::fd::AsRawFd, sync::Arc, thread};

use binder::{binder_impl::IBinderInternal, get_service, SpIBinder, StatusCode};

/// The main entry of this crate
pub struct Dumpsys {
    service: SpIBinder,
}

impl Dumpsys {
    /// Retrieve an existing service and save it for dump, blocking for a few seconds if it doesn't yet exist.
    ///
    /// For example
    ///
    /// ```sh
    /// dumpsys SurfaceFlinger
    /// ```
    ///
    /// is equal to
    ///
    /// ```
    /// use dumpsys_rs::Dumpsys;
    ///
    /// Dumpsys::new("SurfaceFlinger");
    /// ```
    pub fn new<S>(service_name: S) -> Option<Self>
    where
        S: AsRef<str>,
    {
        let service = get_service(service_name.as_ref())?;
        Some(Self { service })
    }

    /// # Example
    ///
    /// ```
    /// use dumpsys_rs::Dumpsys;
    ///
    /// # fn foo() -> Option<()> {
    /// let result = Dumpsys::new("SurfaceFlinger")?
    ///     .dump(&["--latency"])
    ///     .unwrap();
    /// println!("{result}");
    /// # Some(())
    /// # }
    /// ```
    pub fn dump(&self, args: &[&str]) -> Result<String, error::DumpError> {
        let mut buf = String::new();

        {
            let (mut read, write) = os_pipe::pipe()?;
            let handle = thread::spawn(magic(self.service.clone(), write, args));
            let _ = read.read_to_string(&mut buf);
            handle.join().unwrap()?;
        }

        Ok(buf)
    }
}

fn magic(
    mut service: SpIBinder,
    write: impl AsRawFd + Send + 'static,
    args: &[&str],
) -> impl FnOnce() -> Result<(), StatusCode> + Send + 'static {
    let args: Box<[Arc<str>]> = args.into_iter().map(|s| Arc::from(*s)).collect();
    move || {
        let args: Box<[&str]> = args.iter().map(|s| s.as_ref()).collect();
        service.dump(&write, &args)
    }
}
