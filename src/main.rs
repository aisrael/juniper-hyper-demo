use juniper::{self, graphql_value, FieldResult, Variables};
use std::collections::HashMap;
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

fn main() {
    // Create a context object.
    let context = Context::new();
    context.add_user(&User::new("1", "name", "name@example.com"));

    let (_res, _errors) = juniper::execute(
        "mutation createUser { createUser(id: \"2\", name: \"name\", email: \"name@example.com\") { id }}",
        None,
        &Schema::new(Query, Mutation),
        &Variables::new(),
        &context,
    ).unwrap();

    // Run the executor.
    match juniper::execute(
        "query getUser { user(id: \"2\") { id name email } }",
        None,
        &Schema::new(Query, Mutation),
        &Variables::new(),
        &context,
    ) {
        Ok((res, errors)) => {
            println!("{:?}", errors);

            // Ensure the value matches.
            assert_eq!(
                res,
                graphql_value!({
                    "user": {
                        "id": "2",
                        "name": "name",
                        "email": "name@example.com",
                    }
                })
            );
            println!("{:?}", res);
            println!("Ok!");
        }
        Err(e) => {
            println!("{:?}", e);
        }
    }
}
