use futures::future;
use hyper::rt::{self, Future};
use hyper::service::service_fn;
use hyper::Method;
use hyper::{Body, Response, Server, StatusCode};
use juniper::{self, FieldResult};
use std::collections::HashMap;
use std::env;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};

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

fn main() {
    let bind_addr = check_bind_addr_args();

    let context = Arc::new(create_context());
    let root_node = Arc::new(Schema::new(Query, Mutation));

    let new_service = move || {
        let root_node = root_node.clone();
        let context = context.clone();
        service_fn(move |req| -> Box<dyn Future<Item = _, Error = _> + Send> {
            let root_node = root_node.clone();
            let context = context.clone();
            match (req.method(), req.uri().path()) {
                (&Method::GET, "/") => Box::new(juniper_hyper::graphiql("/graphql")),
                (&Method::GET, "/graphql") => {
                    Box::new(juniper_hyper::graphql(root_node, context, req))
                }
                (&Method::POST, "/graphql") => {
                    Box::new(juniper_hyper::graphql(root_node, context, req))
                }
                _ => {
                    let mut response = Response::new(Body::empty());
                    *response.status_mut() = StatusCode::NOT_FOUND;
                    Box::new(future::ok(response))
                }
            }
        })
    };

    let addr = (bind_addr, 3000).into();
    let server = Server::bind(&addr)
        .serve(new_service)
        .map_err(|e| eprintln!("server error: {}", e));
    println!("Listening on http://{}", addr);

    rt::run(server);
}
