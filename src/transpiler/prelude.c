#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include <math.h>
#include <ctype.h>
#include <time.h>
#include <stdarg.h>
#include <assert.h>

#define True true
#define False false
#define Void NULL
#define PI 3.14159265358979323846
#define E 2.71828182845904523536
#define MAX(a, b) ((a) > (b) ? (a) : (b))
#define MIN(a, b) ((a) < (b) ? (a) : (b))
#define ABS(a) ((a) < 0 ? -(a) : (a))

#define len strlen
#define String char*
#define log tsl_log
#define log_int tsl_log_int

#define int_t 0
#define float_t 0.0
#define char_t '\0'
#define bool_t false
#define string_t ""

struct str {
    char* s;
    int len;
    int cap;
};

typedef struct str Str;


Str* newStr() {
    Str* s = (Str*)malloc(sizeof(Str));
    s->s = (char*)malloc(1);
    s->s[0] = '\0';
    s->len = 0;
    s->cap = 1;
    return s;
}

void append(Str* s, char c) {
    if (s->len == s->cap) {
        s->cap *= 2;
        char* old = s->s;
        s->s = (char*)realloc(s->s, s->cap);
        if (s->s == old) {
            free(old);
        }
    }
    s->s[s->len++] = c;
    s->s[s->len] = '\0';
}

void appendStr(Str* s, char* str) {
    for (int i = 0; i < strlen(str); i++) {
        append(s, str[i]);
    }
}

void appendInt(Str* s, int n) {
    char str[12];
    sprintf(str, "%d", n);
    appendStr(s, str);
}

void freeStr(Str* s) {
    free(s->s);
    free(s);
}


void tsl_log(char* s) {
    printf("%s", s);
}

void tsl_log_int(int n) {
    printf("%d", n);
}

char* new_buffer(int size) {
    return (char*)malloc(size);
}

void input(char* buf) {
    scanf("%s", buf);
}

void log_char(char c) {
    printf("%c", c);
}

// ==================== End of prelude.c =================

