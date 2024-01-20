import requests
import websocket
import single_connection_tests
import utils
import dual_connection_tests
import os
import base64
import sys


user1 = {
    "username": "test-user-1-" + utils.get_random_id(),
    "password": "pass"
}

user2 = {
    "username": "test-user-2-" + utils.get_random_id(),
    "password": "pass"
}


def setup_users():
    
    user1_public_key_raw = open(os.path.dirname(os.path.abspath(__file__)) + "/../credentials/test-user-1.pub.pem", "r").read()
    user1_public_key_base64 = base64.b64encode(user1_public_key_raw.encode("ascii")).decode("ascii")
    
    user2_public_key_raw = open(os.path.dirname(os.path.abspath(__file__)) + "/../credentials/test-user-2.pub.pem", "r").read()
    user2_public_key_base64 = base64.b64encode(user2_public_key_raw.encode("ascii")).decode("ascii")

    print(user1_public_key_base64)
    utils.register(user1, user1_public_key_base64)
    utils.register(user2, user2_public_key_base64)

    token1 = utils.login(user1)
    token2 = utils.login(user2)

    friend_request_1_to_2 = utils.send_friend_request(token1, username=user2.get("username"))
    print(friend_request_1_to_2)
    
    uuid = friend_request_1_to_2.get("data").get("id")
    print("friend request id", uuid)
    accepted = utils.accept_friend_request(token2, uuid=uuid)
    print(accepted)


def main():
    utils.debug_version()
    
    setup_users()

    single_connection_tests.perform_single_connection_tests(user1)
    dual_connection_tests.perform_multi_connection_tests(user1=user1, user2=user2)


    print("\n\n\n##################### All tests passed :) #####################")
    exit(0)

main()
















