use futures::Future;
use hyper::header::HeaderValue;
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::Method;
use hyper::{header, Body, Request, Response, Server, StatusCode};
use juniper::{self, FieldResult};
use std::collections::HashMap;
use std::convert::Infallible;
use std::env;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

#[derive(juniper::GraphQLObject, Clone, Debug)]
struct User {
    id: String,
    name: String,
    email: String,
}

impl User {
    fn new<I: Into<String>, N: Into<String>, E: Into<String>>(id: I, name: N, email: E) -> User {
        User {
            id: id.into(),
            name: name.into(),
            email: email.into(),
        }
    }
}

struct Context {
    data: Arc<Mutex<HashMap<String, User>>>,
}

impl Context {
    fn new() -> Context {
        Context {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn add_user(&self, user: &User) {
        self.data
            .lock()
            .unwrap()
            .insert(user.id.clone(), user.clone());
    }

    fn find_user(&self, id: &String) -> Option<User> {
        if let Some(user) = self.data.lock().unwrap().get(id) {
            Some(user.clone())
        } else {
            None
        }
    }
}

impl juniper::Context for Context {}

struct Query;

#[juniper::object(Context = Context)]
impl Query {
    fn user(context: &Context, id: String) -> FieldResult<Option<User>> {
        Ok(context.find_user(&id))
    }
}

struct Mutation;

#[juniper::object(Context = Context)]
impl Mutation {
    fn create_user(
        context: &Context,
        id: String,
        name: String,
        email: String,
    ) -> FieldResult<User> {
        let user = User::new(id, name, email);
        context.add_user(&user);
        Ok(user)
    }
}

// A root schema consists of a query and a mutation.
// Request queries can be executed against a RootNode.
type Schema = juniper::RootNode<'static, Query, Mutation>;

fn create_context() -> Context {
    // Create a context object.
    let context = Context::new();
    context.add_user(&User::new("1", "name", "name@example.com"));
    context
}

fn check_bind_addr_args() -> IpAddr {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let s = &args[1];
        return s.parse::<IpAddr>().unwrap();
    }
    [127, 0, 0, 1].into()
}

async fn handle(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("Hello World\n")))
}

#[tokio::main]
async fn main() {
    let bind_addr = check_bind_addr_args();
    let addr = SocketAddr::new(bind_addr, 3000);

    let context = Arc::new(create_context());
    let root_node = Arc::new(Schema::new(Query, Mutation));

    // And a MakeService to handle each connection...
    let make_service = make_service_fn(|socket: &AddrStream| {
        let c2 = context.clone();
        let r2 = root_node.clone();

        let remote_addr = socket.remote_addr();
        async move {
            let context = c2.clone();
            let root_node = r2.clone();
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| async move {
                let mut r =
                    Response::new(Body::from(juniper::graphiql::graphiql_source("/graphql")));
                *r.status_mut() = StatusCode::OK;
                r.headers_mut().insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("text/html; charset=utf-8"),
                );
                Ok::<_, Infallible>(r)
            }))
        }
    });

    // Then bind and serve...
    let server = Server::bind(&addr).serve(make_service);

    // And run forever...
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
