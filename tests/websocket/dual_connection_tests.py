import websocket
import utils
import constants

def perform_multi_connection_tests(user1, user2):
    token1 = utils.login(user1)
    assert token1 is not None, "Token1 is none, aborting ..."

    token2 = utils.login(user2)
    assert token2 is not None, "Token2 is none, aborting ..."

    connection1 = utils.establish_websocket_connection(token1)
    connection2 = utils.establish_websocket_connection(token2)

    user1_online_users = connection1.recv()
    assert user1_online_users == '{"online_users":["' +user2.get("username") + '"],"TYPE":"SOCKET_MESSAGE_ONLINE_USERS"}', "Incorrect online users"
    print("User1 can see User2 in online users")

    user2_online_users = connection2.recv()
    assert user2_online_users == '{"online_users":["' +user1.get("username") + '"],"TYPE":"SOCKET_MESSAGE_ONLINE_USERS"}', "Incorrect online users"
    print("User2 can see User1 in online users")

    utils.send_and_receive_message("user1_sends_to_user2", connection1, constants.get_testuser1_to_testuser2_message(user2.get("username")), constants.get_expect_testuser1_to_testuser2_message(recipient=user2.get("username"), sender=user1.get("username")))

    user2_received_message_from_user1 = connection2.recv()
    assert user2_received_message_from_user1 == constants.get_expect_testuser1_to_testuser2_receive(recipient=user2.get("username"), sender=user1.get("username")), "User2 did not receive user1 message"
    print("User2 receives messages from user1")

    utils.send_and_receive_message("user2_sends_to_user1", connection2, constants.get_testuser2_to_testuser1_message(recipient=user1.get("username")), constants.get_expect_testuser2_to_testuser1_message(recipient=user1.get("username"), sender=user2.get("username")))

    user1_received_message_from_user2 = connection1.recv()
    assert user1_received_message_from_user2 == constants.get_expect_testuser2_to_testuser1_receive(recipient=user1.get("username"), sender=user2.get("username")), "User2 did not receive user1 message"
    print("User1 receives messages from user2")
