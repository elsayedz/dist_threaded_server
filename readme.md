[For Testing locally]
cargo run  --bin server [Local IP]]:50050 [Local IP]]:50051 [Local IP]]:50052 0

cargo run --bin middleware [Local IP]]:50050 [Local IP]]:50051 [Local IP]]:50052 1

cargo run --bin client

[For Testing on system's lab machines]
cargo run --bin server 10.7.57.87:50050 10.7.57.213:50050 10.7.29.94:50050 0

cargo run --bin middleware 10.7.57.87:50050 10.7.57.213:50050 10.7.29.94:50050