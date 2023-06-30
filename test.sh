#!/bin/bash
SERVER_URI="http://localhost:8080/api/v1"
PASS=0
FAIL=0

check_response() {
    status=$1
    expected_status=$2
    expected_content=$3
    content=$(cat tmp.txt)
    if [[ $status -eq $expected_status && $content == *"$expected_content"* ]]; then
        echo "PASS"
        ((PASS++))
    else
        echo "expected $expected_content got $content"
        echo "FAIL"
        ((FAIL++))
    fi
}

echo "Running setup on the database"
status=$(curl -s -w "%{http_code}" -o tmp.txt --location --request POST "$SERVER_URI/setup")
check_response $status 200 "database: Users-db collection: User-Container"

echo "Looking for Users. This should be empty:"
status=$(curl -s -w "%{http_code}" -o tmp.txt --location "$SERVER_URI/users")
check_response $status 200 "[]"

echo "Creating a user"
status=$(curl -s -w "%{http_code}" -o tmp.txt --location "$SERVER_URI/users" \
--header 'Content-Type: application/x-www-form-urlencoded' \
--data-urlencode 'name=doug' \
--data-urlencode 'email=dougo@test.com')
user=$(cat tmp.txt)
check_response $status 200 "id"

echo "Getting all users again"
status=$(curl -s -w "%{http_code}" -o tmp.txt --location "$SERVER_URI/users")
check_response $status 200 "$user"

echo "Finding one user"
id=$(echo "$user" | jq -r .id)
status=$(curl -s -w "%{http_code}" -o tmp.txt --location "$SERVER_URI/users/$id")
found_user=$(cat tmp.txt)
check_response $status 200 "$user"

echo "Deleting the user"
status=$(curl -s -w "%{http_code}" -o tmp.txt --location --request DELETE "$SERVER_URI/users/$id")
msg=$(cat tmp.txt)
check_response $status 200 "deleted user with id: $id"

echo "PASS: $PASS"
echo "FAIL: $FAIL"
rm tmp.txt 2>/dev/nul