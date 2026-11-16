#ifndef UMATH_MODEL_ABI_H
#define UMATH_MODEL_ABI_H
#include <stddef.h>
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
uint32_t umath_abi_version(void);
/* Buffers remain borrowed until return. JSON is UTF-8 and NUL-terminated.
   The result has {abi_version:1,ok:true,model:{...}} or {ok:false,error:{...}}.
   Free every result exactly once with umath_free_json_v1. */
char *umath_parse_model_json_v1(const uint8_t *input, size_t input_len,
                              const uint8_t *decimal, size_t decimal_len);
/* Additive legacy-route endpoint (the stage producer's route). Same envelope
   and memory contract. Refusals: error.byte_offset is a UTF-8 offset in the
   caller's text, offset_source "caller_input", error.grammar_byte_offset the
   raw grammar-input offset. */
char *umath_parse_model_legacy_json_v1(const uint8_t *input, size_t input_len,
                                       const uint8_t *decimal, size_t decimal_len);
void umath_free_json_v1(char *result);
#ifdef __cplusplus
}
#endif
#endif
