#!/bin/bash -x

curl -H "Content-Type: application/json" \
	-d  @test.json \
	-X POST \
	http://127.0.0.1:8000/secret-store/update
