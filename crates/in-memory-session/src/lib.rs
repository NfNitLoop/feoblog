//! Stores session values in memory, which time out after they
//! haven't been used for some duration.
//!
//! Note: Actix-Web constructs a new service for each server thread.
//! You likely want every thread to have access to the same Session data,
//! so you need to make sure to give them all a reference to the same
//! Middleware instance via an Arc.
//! TODO: 
//! * Document how to properly wrap()
//! * implement timeouts.


// If you find this useful and want to use it, let me know and I'll put it in
// its own crate for you. Otherwise I reserve the right to change its API at
// my whim. :p

// TODO: Consider using parking_lot locks.
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::default::Default;
use std::str::FromStr;
use std::string::ToString;
use std::time::Duration;
use std::ops::{Deref, DerefMut};

use actix_service::{Service, Transform, ServiceExt};
use actix_web::{HttpMessage, FromRequest, HttpRequest};
use actix_web::cookie::Cookie;
use actix_web::dev::{ServiceRequest, ServiceResponse, Payload};
use futures::future::{ok, Future, FutureResult};
use futures::Poll;

use failure::{Error, Fail};

// Cloneable to make use in cloneable closures possible. Makes sharing among
// threads easier.
#[derive(Clone)] 

pub struct Middleware
{
    // This arc is shared among all threads. 
    inner: Arc<MiddlewareInner>
}

// Outer, cloneable container, and public UI.
impl Middleware
{
    pub fn new() -> Self {
        Middleware {
            inner: Arc::new(
                MiddlewareInner {
                    options: Options::default(),
                    sessions: RwLock::new(HashMap::new()),
                }
            )
        }
    }
}

// private
impl Middleware
{
    fn get_or_create_session(&self, req: &ServiceRequest) -> Session
    {
        let register = |session: Session| {
            let extensions = &mut *req.extensions_mut();
            extensions.insert(session.clone());
            session
        };

        let cookie_name = self.inner.options.cookie_name.as_str();
        let cookie = match req.cookie(cookie_name) {
            None => return register(Session::default()),
            Some(c) => c,
        };

        let sessions = self.inner.sessions.read().expect("Poisoned");
        let session = sessions.get(cookie.value()).map(|inner| {
            Session{inner: inner.clone()}
        }).unwrap_or_else(
            || Session::default()
        );
        register(session)
    }

    fn save_session(&self, id: String, inner: Arc<RwLock<SessionInner>>) 
    {
        let mut sessions = self.inner.sessions.write().expect("poisoned");
        sessions.insert(id, inner.clone());
    }

    fn del_session(&self, id: &str)
    {
        let mut sessions = self.inner.sessions.write().expect("poisoned");
        sessions.remove(id);
    }
}

// The bit that's shared among threads via an Arc.
struct MiddlewareInner 
{
    options: Options,
    sessions: RwLock<HashMap<String, Arc<RwLock<SessionInner>>>>,
}

pub struct Options
{
    timeout: Duration,
    cookie_name: String,
}

impl Default for Options
{
    fn default() -> Self {
        Options {
            cookie_name: "session".into(),
            timeout: Duration::from_secs(60*60), // 1h
        }
    }
}

#[derive(Clone)]
pub struct Session
{
    inner: Arc<RwLock<SessionInner>>
}

// Public/User interface:
impl Session
{
    pub fn read(&self) -> impl SessionReader + '_
    {
        SessionReadGuard{ 
            inner: self.inner.read().expect("poisoned"),
        }
    }

    pub fn write(&self) -> impl SessionWriter + '_
    {
        SessionWriteGuard{
            inner: self.inner.write().expect("poisoned")
        }
    }
}

/// 1) Acts as a read lock on the session. (Free it when you're done!)
/// 2) Provides access to the underlying HashMap<String,String>
/// 3) Provides helper functions for (de)serializing to/from the map.
pub trait SessionReader: Deref<Target=HashMap<String,String>>
{
    /// Get a value from the session, parsing it to the destination type.
    /// Errors parsing will return None. If you need to distinguish between
    /// empty/unparseable, use the underlying HashMap methods.
    fn get<T>(&self, key: &str) -> Option<T>
    where T: FromStr
    {
        let value = match self.deref().get(key) {
            None => return None,
            Some(v) => v
        };
        FromStr::from_str(value.as_str()).ok()
    }
}

impl<T> SessionReader for T
where T: Deref<Target=HashMap<String,String>>
{

}

pub trait SessionWriter: SessionReader + DerefMut
{
    /// Set a value to the session, converting it to a string automatically.
    /// also may allocate a new String key. Because HashMap.
    fn set<T>(&mut self, key: &str, value: T)
    where T: ToString
    {
        self.deref_mut().insert(key.to_string(), value.to_string());
    }
}

impl SessionWriter for SessionWriteGuard<'_>
{

}

// TODO: Could use parking_lot instead of writing my own:
struct SessionReadGuard<'a>
{
    inner: std::sync::RwLockReadGuard<'a, SessionInner>,
}

impl<'a> Deref for SessionReadGuard<'a>
{
    type Target = HashMap<String, String>;
    
    fn deref(&self) -> &Self::Target {
        &self.inner.data
    }
}

struct SessionWriteGuard<'a>
{
    inner: std::sync::RwLockWriteGuard<'a, SessionInner>,
}

impl<'a> Deref for SessionWriteGuard<'a>
{
    type Target = HashMap<String, String>;
    
    fn deref(&self) -> &Self::Target {
        &self.inner.data
    }
}

impl<'a> DerefMut for SessionWriteGuard<'a>
{    
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner.data
    }
}

// private:
impl Session
{
    fn finish<B: 'static>(
        self,
        res: ServiceResponse<B>,
        middleware: Middleware,
    ) -> ServiceResponse<B>
     {
        use SessionState::*;
        let cookie_name = middleware.inner.options.cookie_name.as_str();
        let mut res = res;
        let hres = res.response_mut();
        let mut session_inner = self.inner.write().expect("poiosoned");
        let session_id = match &session_inner.state {
            Existing{session_id} => session_id.to_string(),
            DeleteMe{session_id} => session_id.to_string(),
            New => {
                use rust_sodium::randombytes::randombytes;
                use rust_base58::*;
                let session_id = randombytes(20); // same as sha1.
                session_id[..].to_base58()
            }
        };

        let mut write_cookie = || {
            let cookie = Cookie::build(cookie_name.to_string(), session_id.clone())
                .path("/")
                .max_age(middleware.inner.options.timeout.as_secs() as i64)
                .finish()
            ;
            hres.add_cookie(&cookie).unwrap();
        };

        match &session_inner.state {
            New => {
                if session_inner.data.is_empty() {
                    return res; // No need to save session.
                }
                // TODO: DO I really need to save the ID here?
                session_inner.state = Existing{ session_id: session_id.clone()};
                middleware.save_session(session_id.clone(), Arc::clone(&self.inner));
                write_cookie();
            },
            DeleteMe{session_id} => {
                // TODO: Actually, we probably want to send a tmp cookie to
                // unset from the browser. This just deletes locally.
                hres.del_cookie(cookie_name);
                middleware.del_session(session_id.as_str());
            },
            Existing{session_id:_} => {
                // session is already tracked in middleware.
                // always (re)write the cookie to extend its duration:
                write_cookie();
            }
        }
        res
    }
}

impl FromRequest for Session {
    type Error = Error;
    type Future = Result<Session, Error>;
    type Config = ();

    #[inline]
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let mut extensions = &*req.extensions_mut();
        extensions.get::<Session>()
            .map(|s| s.clone())
            .ok_or( 
                NoSessionFound.into()
            )
    }
}

#[derive(Debug, Fail)]
#[fail(display="No session found. Did you forget to install the middleware?")]
struct NoSessionFound;

impl Default for Session
{
    // A session that's not even saved anywhere yet. 
    fn default() -> Self { 
        let inner = SessionInner{
            data: HashMap::new(),
            state: SessionState::New,
        };
        let inner = RwLock::new(inner);
        let inner = Arc::new(inner);

        Session{inner}
    }
}

struct SessionInner
{
    data: HashMap<String,String>,
    state: SessionState
}

enum SessionState {
    /// Session just created.
    New,

    /// Session was loaded from user cookie.
    Existing { session_id: String },

    /// Session is marked for deletion.
    DeleteMe { session_id: String },
}

impl<S, B: 'static> Transform<S> for Middleware
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>>,
    S::Future: 'static,
    S::Error: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = S::Error;
    type InitError = ();
    type Transform = SessionService<S>;
    type Future = FutureResult<Self::Transform, Self::InitError>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(SessionService {
            service,
            middleware: self.clone(),
        })
    }
}


pub struct SessionService<S>
{
    service: S,
    middleware: Middleware,
}

impl<S, B: 'static> Service for SessionService<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>>,
    S::Future: 'static,
    S::Error: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = S::Error;
    type Future = Box<dyn Future<Item = Self::Response, Error = Self::Error>>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        self.service.poll_ready()
    }

    fn call(&mut self, mut req: ServiceRequest) -> Self::Future {
        let middleware = self.middleware.clone();
        let session = middleware.get_or_create_session(&req);
        
        Box::new(self.service.call(req).map(move |mut res| {
            session.finish(res, middleware) // TODO middleware.
        }))
    }
}

