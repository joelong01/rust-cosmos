#!/bin/bash
SERVER_URI="http://localhost:8080"
echo "running setup on the database"
curl -s --location --request POST "$SERVER_URI/api/v1/setup"
echo "looking for Users.  This should be empty:"
curl -s --location 'localhost:8080/api/v1/users' | jq .
echo "Creating a user"
user=$(curl -s --location "$SERVER_URI/api/v1/users' \
--header 'Content-Type: application/x-www-form-urlencoded' \
--data-urlencode 'name=doug' \
--data-urlencode 'email=dougo@test.com")
echo "the return from the create_user. this should NOT be empty"
echo "$user" | jq .
echo "getting all users again"
curl -s --location "$SERVER_URI/api/v1/users" | jq .
echo "finding one user"
id=$(echo "$user" | jq -r .id)
found_user=$(curl -s --location "$SERVER_URI/api/v1/users/$id")
echo "found: "
echo "$found_user"
echo "deleting the user"
msg=$(curl -s --location --request DELETE "$SERVER_URI/api/v1/users/$id")
echo "$msg"