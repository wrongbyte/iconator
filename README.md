# Overview
This is an API that returns an SVG icon given a path as the input. It currently has only one route: GET /icons with two query params: `type` and `path`.
```
GET /icon/?type=file&path=.gitignore
```
 
## How to run
### Cargo
To run this service, simply run `cargo run` in the root of the repository.
### Docker
Run
```
docker build -t iconator .
docker run --rm -p 3000:3000 iconator
```
 
## Design
To write this web server, I chose Axum because it is simple and fast. It provides extractors to get the data received in the request, as well as a saner way to deal with application state - which is empty here, but can be easily extended on demand.
Other third-party crates used include `thiserror`, which provides an ergonomic way to declare and use errors, `tracing` for structured logs, and the `TraceLayer` middleware from `tower-http` for logging request information.
 
For larger projects, a more robust module structure can be used; however, I chose to keep it simple for now and declare the route functions in `main.rs`.

### Security
This service has a relatively small security surface because it does not search for files in the filesystem or execute user-provided paths. The requested icon name is only used as an identifier to match known/static icon data, which avoids common risks such as path traversal or arbitrary file reads.

That said, security should be reconsidered if the service scope expands. If future versions load icons from disk, accept custom paths, fetch remote resources, or generate files dynamically, the input should be strictly validated and constrained to an allowed set of values.

Axum’s extractors handle URL parsing/decoding, so encoded route parameters are normalized before reaching the handler. However, this should not be treated as complete sanitization. The application should still validate the decoded value, reject unexpected characters, enforce length limits, and avoid using user input directly in filesystem paths, shell commands, or external requests.

Additional hardening could include rate limiting, request size limits, structured error responses that do not leak internals, dependency scanning, running the container as a non-root user, and serving only the intended content types.
 
# Changes in `iconator` library
The original `iconator` library code had extensive usage of `unwrap()`, compromising its usability. Since `get_icon_for_file` and `get_icon_for_folder` are fallible, the ideal return type for them is a `Result` with common errors mapped exhaustively - namely, through an enum.
Therefore, I chose to change the lib code in order to remove the `unwrap()`s in the public methods, create and export an error type, and add a bit of documentation to its methods.
Additionally, more tests were added. Since the original code relied on panics for invalid input, test cases included only happy paths. This makes it harder to reason about how the code can fail, so X more tests were added to cover the most common scenarios.
 
I tried to make the changes as minimal as possible and avoid any changes to the existing logic in the library, but I would like to mention that `unwrap_unchecked` is possibly another problematic usage, since it imposes the (possibly unnecessary in our case) risk of undefined behavior.
 
### Alternatives
I considered not changing the library for a while; however, it would require the server to duplicate validations - for example, since `std::path::Path::file_name` is fallible and called in both library public methods, the route would need to call it, verify its result, and only then call the corresponding library method. The same goes for `.extension()`. Additionally, I would need to map in the server all the errors that should belong to the lib realm. Not good!
 
# Next steps
The code is extensible: new errors can be mapped exhaustively in `ApplicationError`, application state can be added to `AppState` to be shared across the server, and new routes can be added to the main router.

To productionize this service, it can be packaged as a Docker image and deployed behind a load balancer, allowing multiple instances to run concurrently. A Kubernetes deployment would make it possible to scale replicas based on CPU, memory, or request volume, while also providing rolling updates, health checks, and automatic restarts. The Docker image itself can be optimized for a smaller size and to take advantage of caching.
