cargo run  192.168.1.13:50050 192.168.1.13:50051 192.168.1.13:50052 0

[For Testing locally]
cargo run  --bin server 10.40.59.70:50050 10.40.59.70:50051 10.40.59.70:50052 0

cargo run --bin middleware 10.40.59.70:50050 10.40.59.70:50051 10.40.59.70:50052 1

cargo run --bin client

[For Testing on system's lab machines]
cargo run --bin server 10.7.57.87:50050 10.7.57.213:50050 10.7.29.94:50050 0

cargo run --bin middleware 10.7.57.87:50050 10.7.57.213:50050 10.7.29.94:50050