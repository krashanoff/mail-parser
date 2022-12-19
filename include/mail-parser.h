#ifndef MAIL_PARSER_H_
#define MAIL_PARSER_H_

struct Message;

int parse_message(Message **message_out, char *message);
int free_message(Message **message);

#endif // MAIL_PARSER_H_
