use juniper::{self, graphql_value, EmptyMutation, FieldResult, Variables};

struct Ctx {}

impl juniper::Context for Ctx {}

#[derive(juniper::GraphQLObject)]
struct Person {
    id: String,
    name: String,
    age: i32,
}

struct Query;

#[juniper::object]
impl Query {
    fn person() -> FieldResult<Option<Person>> {
        Ok(Some(Person {
            id: "1".into(),
            name: "name".into(),
            age: 23,
        }))
    }
}

// A root schema consists of a query and a mutation.
// Request queries can be executed against a RootNode.
type Schema = juniper::RootNode<'static, Query, EmptyMutation<()>>;

fn main() {
    // Create a context object.
    let ctx = Ctx {};

    // Run the executor.
    let (res, _errors) = juniper::execute(
        "query { person { id name age } }",
        None,
        &Schema::new(Query, EmptyMutation::new()),
        &Variables::new(),
        &(),
    )
    .unwrap();

    // Ensure the value matches.
    assert_eq!(
        res,
        graphql_value!({
            "person": {
                "id": "1",
                "name": "name",
                "age": 23,
            }
        })
    );
    println!("{:?}", res);
    println!("Ok!");
}
