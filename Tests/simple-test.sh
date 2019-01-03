#!/bin/bash

qs='host=wicci.org&user=greg@wicci.org'
page="simple.html"
url="http://localhost:8080/$page?$qs"

curl "$url" -v | sed -e '/^< /!d' -e 's/^< //' diff
