cargo run  10.65.192.142:50050 10.65.192.142:50051 10.65.192.142:50052 0

[For Testing locally]
cargo run  --bin server 10.40.37.143:50050 10.40.37.143:50051 10.40.37.143:50052 0

cargo run --bin middleware 10.40.37.143:50050 10.40.37.143:50051 10.40.37.143:50052

cargo run --bin client

[For Testing on system's lab machines]
cargo run --bin server 10.7.57.87:50050 10.7.57.213:50050 10.7.29.94:50050 0

cargo run --bin middleware 10.7.57.87:50050 10.7.57.213:50050 10.7.29.94:50050