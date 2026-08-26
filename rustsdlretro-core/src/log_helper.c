#include <stdarg.h>
#include <stdio.h>

/* C helper that expands variadic format string and calls Rust handler.
 * The core (e.g. Beetle PSX) calls this via retro_log_printf with %s/%d etc.
 * We use vsnprintf to fully expand the format, then pass the plain string to Rust.
 */

extern void rust_log_callback(unsigned level, const char* message);

#define MAX_LOG 4096

void __attribute__((visibility("default"))) rustsdlretro_log_handler(unsigned level, const char* fmt, ...) {
    va_list ap;
    char buf[MAX_LOG];
    va_start(ap, fmt);
    vsnprintf(buf, sizeof(buf), fmt, ap);
    va_end(ap);
    rust_log_callback(level, buf);
}
