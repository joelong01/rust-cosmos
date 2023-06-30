#!/bin/bash

PASS=0
FAIL=0
#!/bin/bash

# Initialize defaults
SERVER_URI="http://localhost:8080/api/v1"
VERBOSE=false

# Parse command-line arguments
while getopts ":u:v" opt; do
  case ${opt} in
    u )
      SERVER_URI=$OPTARG
      ;;
    v )
      VERBOSE=true
      ;;
    \? )
      echo "Invalid Option: -$OPTARG" 1>&2
      exit 1
      ;;
    : )
      echo "Invalid Option: -$OPTARG requires an argument" 1>&2
      exit 1
      ;;
  esac
done
shift $((OPTIND -1))

# Remainder of your script goes here...

check_response() {
    status=$1
    expected_status=$2
    expected_content=$3
    content=$(cat tmp.txt)
    if [[ "$status" -eq $expected_status && $content == *"$expected_content"* ]]; then
        echo "PASS"
        ((PASS++))
    else
        echo "expected $expected_content got $content"
        echo "FAIL"
        ((FAIL++))
    fi
}
echo_if_verbose() {
    if [[ $VERBOSE ]]; then 
        echo "$1"
    fi
}
echo "Running setup on the database"
status=$(curl -s -w "%{http_code}" -o tmp.txt --location --request POST "$SERVER_URI/setup")
check_response "$status" 200 "database: Users-db collection: User-Container"
echo_if_verbose "$(jq . < tmp.txt)"

echo "Looking for Users. This should be empty:"
status=$(curl -s -w "%{http_code}" -o tmp.txt --location "$SERVER_URI/users")
check_response "$status" 200 "[]"
echo_if_verbose "$(jq . < tmp.txt)"

echo "Creating a user"
status=$(curl -s -w "%{http_code}" -o tmp.txt --location "$SERVER_URI/users" \
--header 'Content-Type: application/x-www-form-urlencoded' \
--data-urlencode 'name=doug' \
--data-urlencode 'email=dougo@test.com')
user=$(cat tmp.txt)
check_response "$status" 200 "id"
echo_if_verbose "$(jq . < tmp.txt)"

echo "Getting all users again"
status=$(curl -s -w "%{http_code}" -o tmp.txt --location "$SERVER_URI/users")
check_response "$status" 200 "$user"
echo_if_verbose "$(jq . < tmp.txt)"
echo "Finding one user"
id=$(echo "$user" | jq -r .id)
status=$(curl -s -w "%{http_code}" -o tmp.txt --location "$SERVER_URI/users/$id")
found_user=$(cat tmp.txt)
echo_if_verbose "$found_user \n $status"
echo_if_verbose "$(jq . < tmp.txt)"

echo "Deleting the user"
status=$(curl -s -w "%{http_code}" -o tmp.txt --location --request DELETE "$SERVER_URI/users/$id")
check_response "$status" 200 "deleted user with id: $id"
echo_if_verbose "$(jq . < tmp.txt)"

echo "PASS: $PASS"
echo "FAIL: $FAIL"
rm tmp.txt 2>/dev/null