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

    expect = '{"online_users":["' +user2.get("username") + '"],"TYPE":"SOCKET_MESSAGE_ONLINE_USERS"}'
    utils.receive_message("User1 sees online users", connection=connection1, expect=expect)

    expect = '{"online_users":["' +user1.get("username") + '"],"TYPE":"SOCKET_MESSAGE_ONLINE_USERS"}'
    utils.receive_message("User2 sees online users", connection=connection2, expect=expect)

    utils.send_and_receive_message("user1_sends_to_user2", connection1, constants.get_testuser1_to_testuser2_message(user2.get("username")), constants.get_expect_testuser1_to_testuser2_message(recipient=user2.get("username"), sender=user1.get("username")))


    expect = constants.get_expect_testuser1_to_testuser2_receive(recipient=user2.get("username"), sender=user1.get("username"))
    utils.receive_message("User2 receives messages from user1", connection=connection2, expect=expect)


    utils.send_and_receive_message("user2_sends_to_user1", connection2, constants.get_testuser2_to_testuser1_message(recipient=user1.get("username")), constants.get_expect_testuser2_to_testuser1_message(recipient=user1.get("username"), sender=user2.get("username")))

    expect = constants.get_expect_testuser2_to_testuser1_receive(recipient=user1.get("username"), sender=user2.get("username"))
    utils.receive_message("User2 receives messages from user1", connection=connection1, expect=expect)

    connection1.close()
    connection2.close()
