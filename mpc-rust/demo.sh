#!/bin/bash
# 実プロセス版MPCのデモ: node(3プロセス) + participant(5プロセス、1社1プロセス) + coordinate(公開データ集計)
set -e

cd "$(dirname "$0")"

echo "== build =="
cargo build --bin tcp_server --bin participant --bin cordinate

echo "== start 3 nodes (ports 8000-8002) =="
./target/debug/tcp_server 8000 > /tmp/mpc_demo_node8000.log 2>&1 &
NODE0=$!
./target/debug/tcp_server 8001 > /tmp/mpc_demo_node8001.log 2>&1 &
NODE1=$!
./target/debug/tcp_server 8002 > /tmp/mpc_demo_node8002.log 2>&1 &
NODE2=$!

cleanup() {
    echo "== stopping nodes =="
    kill "$NODE0" "$NODE1" "$NODE2" 2>/dev/null
}
trap cleanup EXIT

sleep 1

echo "== 5 participants send their own share (each process knows only its own data) =="
# Buyer: b1, b2, b3 / Seller: s1, s2
./target/debug/participant true  100 100   # b1: threshold=100, quantity=100
./target/debug/participant true  110 200   # b2: threshold=110, quantity=200
./target/debug/participant true  105 300   # b3: threshold=105, quantity=300
./target/debug/participant false 90  400   # s1: threshold=90,  quantity=400
./target/debug/participant false 100 500   # s2: threshold=100, quantity=500

echo "== coordinate: collect public data only (clearing price, matched quantity) =="
./target/debug/cordinate
