#  for loop 10 times and run command
for i in {1..500}
do
    cargo run  --bin client $i &
done