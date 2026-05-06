#pragma once

#include "tokenizer.hpp"
#include <ctranslate2/translator.h>

#ifdef __cplusplus
extern "C" {
#endif

#define JNI_SUPPORT // comment this out if you don't want JNI.

#ifdef JNI_SUPPORT
#include <jni.h>
#endif

#if defined(_WIN32)
#define __export __declspec(dllexport)
#elif defined(__GNUC__) && ((__GNUC__ >= 4) || (__GNUC__ == 3 && __GNUC_MINOR__ >= 3))
#define __export __attribute__((visibility("default")))
#else
#define __export
#endif

__export struct UnityTranslateLibInstance {
    Tokenizer* tokenizer;
    ctranslate2::Translator* translator;
};

__export enum TokenizerType {
    SENTENCEPIECE, BPE
};

/**
 * Used for creating a new UnityTranslateLib instance.
 */
__export UnityTranslateLibInstance* createInstance(const char* toLang, const char* translatorModelPath, TokenizerType type, const char* tokenizerModelPath, bool useCuda);

/**
 * Translates all the provided text, in batch. The results array is for output, and must match the provided array length.
 * This is the C-compatible variant of batchTranslate.
 */
__export void batchTranslate(const UnityTranslateLibInstance* instance, int arrayLength, const char** textToTranslate, const char** results);

/**
 * Used for freeing the UnityTranslateLib instance pointers. Must be called for closing the translator.
 */
__export void freeInstance(UnityTranslateLibInstance* instance);

#ifdef JNI_SUPPORT
JNIEXPORT jlong JNICALL Java_xyz_bluspring_unitytranslate_library_UnityTranslateLib_createInstance(JNIEnv* env, jobject thisObj, jstring toLang, jstring translatorModelPath, jint type, jstring tokenizerModelPath, jboolean useCuda);
JNIEXPORT void JNICALL Java_xyz_bluspring_unitytranslate_library_UnityTranslateLib_batchTranslate(JNIEnv* env, jobject thisObj, jlong instance, jobjectArray textToTranslate, jobjectArray results);
JNIEXPORT void JNICALL Java_xyz_bluspring_unitytranslate_library_UnityTranslateLib_freeInstance(JNIEnv* env, jobject thisObj, jlong instance);
#endif

#ifdef __cplusplus
}
#endif

/**
 * Translates all the provided text, in batch. The results vector is for output, and must match the textToTranslate length.
 * This is the C++ compatible variant of batchTranslate.
 */
__export void batchTranslate(const UnityTranslateLibInstance* instance, const vector<string>& textToTranslate, vector<string> results);