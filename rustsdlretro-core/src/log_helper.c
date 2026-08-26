#include <stdarg.h>
#include <stdio.h>

/* C shim for variadic retro_log_printf_t callback.
 * The core calls this with (level, fmt, ...). We format the message
 * using vsnprintf and forward to rust_log_callback(Rust) with plain string. */

extern void rust_log_callback(unsigned level, const char* msg);

void __attribute__((visibility("default"))) rustsdlretro_log_handler(unsigned level, const char* fmt, ...) {
    char buf[4096];
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(buf, sizeof(buf), fmt, ap);
    va_end(ap);
    
    rust_log_callback(level, buf);
}
