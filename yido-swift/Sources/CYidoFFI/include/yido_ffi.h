#ifndef YIDO_FFI_H
#define YIDO_FFI_H

/* Generated with cbindgen:0.29.3 */

#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct YidoEngine {
  uint8_t _private[0];
} YidoEngine;

typedef struct YidoEngineCreateResult {
  struct YidoEngine *engine;
  char *error;
} YidoEngineCreateResult;

typedef struct YidoInputEffect {
  char *committed;
  char *composing;
  bool handled;
  char *error;
} YidoInputEffect;

struct YidoEngineCreateResult yido_engine_new(const char *layout_toml);

struct YidoInputEffect yido_engine_input_key(struct YidoEngine *engine,
                                             const char *key,
                                             bool shift);

struct YidoInputEffect yido_engine_backspace(struct YidoEngine *engine);

struct YidoInputEffect yido_engine_flush(struct YidoEngine *engine);

struct YidoInputEffect yido_engine_cancel(struct YidoEngine *engine);

void yido_engine_create_result_free(struct YidoEngineCreateResult result);

void yido_input_effect_free(struct YidoInputEffect effect);

void yido_engine_free(struct YidoEngine *engine);

#endif  /* YIDO_FFI_H */
