juniper-hyper-demo
====

An example project built in Rust demonstrating [juniper](https://github.com/graphql-rust/juniper) and [hyper](https://github.com/hyperium/hyper).

### To run

```
$ cargo run
  .
  .
  .
Listening on http://127.0.0.1:3000
```

Or, alternatively, to build and run a release build

```
$ cargo run --release
  .
  .
  .
Listening on http://127.0.0.1:3000
```

### To test

```
$ curl -X POST -d @request.json http://localhost:3000/graphql
{
  "data": {
    "user": {
      "name": "name",
      "email": "name@example.com"
    }
  }
}
```

### To benchmark

Using Apache bench, [ab](https://httpd.apache.org/docs/2.4/programs/ab.html):

```
$ ab -p request.json -c 10 -n 1000 127.0.0.1:3000/graphql
```

### Building the Docker image

```
$ docker build -t juniper-hyper-demo .
```

(Warning: takes about 10-15 minutes.)
