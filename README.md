## Setup Instructions

1. Postgres should be installed and running.
2. Rust and cargo should be installed on the system.
3. Edit .env file in project root and update your postgres credentials there. (No need to create 'rust_ecommerce' database manually as it would be creted by diesel setup.)
4. Install diesel cli by running command as below in the project- 

```sh
# Install diesel cli for postgres
cargo install diesel_cli --no-default-features --features postgres
```
5. Run the diesel setup command as below. It will create db and run migrations so that all tables and indexes are created.
```sh
diesel setup
```
7. Start the project with cargo as 
```
cargo run
```


## Notes
* 'ecommerce_demo.postman_collection.json' file is present at the root of the project. It is postman export file and has all the apis to test if one wants to use Postman. The postman export format is v2.1
* 'created_at' values in tables could be have been generated by default in postgres itself but I am doing it in code.
* I was facing some issue with diesel types so I created 'qty' and 'price' columns in order_items table of type integer instead of numeric.
* User password is stored as plain text since this is just demostration project. Otherwise it should always be encrepted with proper strategy.
* 'registration' and 'login' routes return jwt token in response body. This token should be passed in request header 'access_token' for authenticated routes.
* Better actix route registration/mounting could have been used. But doing plain route registration here.
* Logging could have been better, but again this is demo exercise.