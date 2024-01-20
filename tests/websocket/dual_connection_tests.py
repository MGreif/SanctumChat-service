import websocket
import utils
import constants
import json


def perform_multi_connection_tests(user1, user2):
    token1 = utils.login(user1)
    assert token1 is not None, "Token1 is none, aborting ..."

    token2 = utils.login(user2)
    assert token2 is not None, "Token2 is none, aborting ..."

    connection1 = utils.establish_websocket_connection(token1)
    connection2 = utils.establish_websocket_connection(token2)

    expect = '{"online_users":["' +user2.get("username") + '"],"TYPE":"SOCKET_MESSAGE_ONLINE_USERS"}'
    utils.receive_message("User1 sees online users", connection=connection1, expect=expect)

    expect = '{"online_users":["' +user1.get("username") + '"],"TYPE":"SOCKET_MESSAGE_ONLINE_USERS"}'
    utils.receive_message("User2 sees online users", connection=connection2, expect=expect)

    connection1.send(constants.get_testuser1_to_testuser2_message(user2.get("username")))
    received = connection1.recv()
    received_json = json.loads(received)
    expect = constants.get_expect_testuser1_to_testuser2_message(id=received_json["id"], recipient=user2.get("username"), sender=user1.get("username"))
    utils.compare("user1_sends_to_user2", received, expect=expect)


    expect = constants.get_expect_testuser1_to_testuser2_receive(id=received_json["id"], recipient=user2.get("username"), sender=user1.get("username"))
    utils.receive_message("User2 receives messages from user1", connection=connection2, expect=expect)

    connection2.send(constants.get_testuser2_to_testuser1_message(user1.get("username")))
    received = connection2.recv()
    received_json = json.loads(received)
    expect = constants.get_expect_testuser2_to_testuser1_message(id=received_json["id"], recipient=user1.get("username"), sender=user2.get("username"))
    utils.compare("user2_sends_to_user1", received, expect=expect)

    
    expect = constants.get_expect_testuser2_to_testuser1_receive(id=received_json["id"], recipient=user1.get("username"), sender=user2.get("username"))
    utils.receive_message("User2 receives messages from user1", connection=connection1, expect=expect)

    connection1.close()
    connection2.close()
