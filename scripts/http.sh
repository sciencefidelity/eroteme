#!/bin/sh

# curl -X OPTIONS localhost:3030/questions \
#       -H "Access-Control-Request-Method: PUT" \
#       -H "Access-Control-Request-Headers: content-type" \
#       -H "Origin: https:/ /not-origin.io" -verbose

# curl --location --request POST 'localhost:3030/questions' \
#      --header 'Content-Type: application/json' \
#      --data-raw '{
#         "title": "New question",
#         "content": "How does this work again?”
#      }' \

curl -H 'Content-Type: application/json' \
      -d @$2.json \
      -X POST \
      -s \
      http://localhost:3030/$1 \
     # | jq '.'
