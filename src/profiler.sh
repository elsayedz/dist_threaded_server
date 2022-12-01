#  To run 500 client, use this script
for i in {1..500}
do
    cargo run  --bin client $i &
done