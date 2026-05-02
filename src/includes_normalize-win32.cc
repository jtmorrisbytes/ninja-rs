// Copyright 2012 Google Inc. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#include <windows.h>

#include <algorithm>
#include <iterator>
#include <sstream>
#include <string>
#include <vector>

#include "includes_normalize.h"
#include "string_piece.h"
#include "string_piece_util.h"
#include "util.h"

using namespace std;

// we are ignoring 'long paths in registry'
// and attempting to use the W variant of windows apis
#ifndef WIN32_WIDE_PATH_LIMIT
#define WIN32_WIDE_PATH_LIMIT 32767
#endif

#ifndef RS_CANNON_PATH_
#define RS_CANNON_PATH_
extern "C" void write_to_cpp_string(void* ctx, const char* data) {
  // This is safe! It uses the C++ compiler's string logic.
  static_cast<std::string*>(ctx)->assign(data);
}

typedef void (*Abs2Cb)(void* ctx, const char* data);
extern "C" {
char* rs_canonicalize_path2(const char* path, uint64_t* slash_bits);
char* rs_canonicalize_path3(const char* path, size_t* len,
                            uint64_t* slash_bits);
// this one uses a cb pattern
// takes path as input, string as output, then writes the lexical absolute path using cb
// when possible. 
void rs_abs_path2(const char* path, void* string_ctx, Abs2Cb cb);
bool rs_is_same_lexical_drive(const char* a, const char* b);
}
#endif

namespace {
bool InternalGetFullPathName(const StringPiece& file_name, string buffer,
                             size_t buffer_length, string* err) {
  // 1. Convert the 8-bit ANSI string to a 16-bit Wide string (Unicode)
  std::string input = file_name.AsString();
  int wlen = MultiByteToWideChar(CP_ACP, 0, input.c_str(), -1, NULL, 0);
  std::wstring winput(wlen, 0);
  MultiByteToWideChar(CP_ACP, 0, input.c_str(), -1, &winput[0], wlen);

  // 2. Add the magic prefix for long paths if it's not already there
  if (winput.find(L"\\\\?\\") != 0) {
    winput = L"\\\\?\\" + winput;
  }

  // 3. Use the UNICODE version of the utility
  DWORD result_size = GetFullPathNameW(winput.c_str(), 0, NULL, NULL);
  if (result_size == 0) {
    *err = "GetFullPathNameW failed: " + GetLastErrorString();
    return false;
  }

  // 4. Get the full wide path
  std::wstring wresult(result_size, 0);
  GetFullPathNameW(winput.c_str(), result_size, &wresult[0], NULL);

  // 5. Convert it BACK to ANSI so the rest of Ninja doesn't have a heart attack
  // Note: This is where we might use GetShortPathNameA if the path is truly
  // massive
  int alen = WideCharToMultiByte(CP_ACP, 0, wresult.c_str(), -1, buffer.data(),
                                 (int)buffer.size(), NULL, NULL);

  if (alen == 0) {
    // If it doesn't fit in the buffer, this is where we'd use a
    // specialized short-path (8.3) fallback to keep the 260 limit happy.
    *err = "Path still exceeds buffer after normalization";
    return false;
  }

  return true;
}

// bool InternalGetFullPathName(const StringPiece& file_name, char* buffer,
//                              size_t buffer_length, string *err) {
//   DWORD result_size = GetFullPathNameA(file_name.AsString().c_str(),
//                                        buffer_length, buffer, NULL);
//   if (result_size == 0) {
//     *err = "GetFullPathNameA(" + file_name.AsString() + "): " +
//         GetLastErrorString();
//     return false;
//   } else if (result_size > buffer_length) {
//     *err = "path too long";
//     return false;
//   }
//   return true;
// }

bool IsPathSeparator(char c) {
  return c == '/' || c == '\\';
}


// Return true if paths a and b are on the same Windows drive.
bool SameDrive(StringPiece a, StringPiece b, string* err) {
  // sigh ironically another rust function
  // checks for \\?\C:\ C:\ and \\ using rust
   return rs_is_same_lexical_drive(a.AsString().c_str(),b.AsString().c_str());
}

// Check path |s| is FullPath style returned by GetFullPathName.
// This ignores difference of path separator.
// This is used not to call very slow GetFullPathName API.
bool IsFullPathName(StringPiece s) {
  if (s.size() < 3 || !islatinalpha(s[0]) || s[1] != ':' ||
      !IsPathSeparator(s[2])) {
    return false;
  }

  // Check "." or ".." is contained in path.
  for (size_t i = 2; i < s.size(); ++i) {
    if (!IsPathSeparator(s[i])) {
      continue;
    }

    // Check ".".
    if (i + 1 < s.size() && s[i + 1] == '.' &&
        (i + 2 >= s.size() || IsPathSeparator(s[i + 2]))) {
      return false;
    }

    // Check "..".
    if (i + 2 < s.size() && s[i + 1] == '.' && s[i + 2] == '.' &&
        (i + 3 >= s.size() || IsPathSeparator(s[i + 3]))) {
      return false;
    }
  }

  return true;
}

}  // anonymous namespace

IncludesNormalize::IncludesNormalize(const string& relative_to) {
  // ?????
  string err;
  relative_to_ = AbsPath(relative_to, &err);
  if (!err.empty()) {
    Fatal("Initializing IncludesNormalize(): %s", err.c_str());
  }
  split_relative_to_ = SplitStringPiece(relative_to_, '/');
}

string IncludesNormalize::AbsPath(StringPiece s, string* err) {
  // decided to replace this with rust std::fs::absolute
  // doesnt touch disk every time but will handle VERY long paths
  // unless you want to malloc

  // c_string input requred
  string output = "";
  string input = s.AsString();
  rs_abs_path2(input.c_str(), &output, write_to_cpp_string);
  if (output == "") {
    *err = "Failed to get abs path";
  }
  return output;
  // err check
}

string IncludesNormalize::Relativize(StringPiece path,
                                     const vector<StringPiece>& start_list,
                                     string* err) {
  printf_s("Relativize %s\n",path.AsString().c_str());
  string abs_path = AbsPath(path, err);
  if (!err->empty())
    return "";
  vector<StringPiece> path_list = SplitStringPiece(abs_path, '/');
  int i;
  for (i = 0; i < static_cast<int>(min(start_list.size(), path_list.size()));
       ++i) {
    if (!EqualsCaseInsensitiveASCII(start_list[i], path_list[i])) {
      break;
    }
  }

  vector<StringPiece> rel_list;
  rel_list.reserve(start_list.size() - i + path_list.size() - i);
  for (int j = 0; j < static_cast<int>(start_list.size() - i); ++j)
    rel_list.push_back("..");
  for (int j = i; j < static_cast<int>(path_list.size()); ++j)
    rel_list.push_back(path_list[j]);
  if (rel_list.size() == 0)
    return ".";
  return JoinStringPiece(rel_list, '/');
}
extern "C" {
  bool rs_includes_normalize(const char* input,const char* relative_to,char** output,char** c_err);
}

bool IncludesNormalize::Normalize(const string& input, string* result,
                                  string* err) const {
  

  char* r = nullptr;
  char* c_err = nullptr;
  bool b = rs_includes_normalize(input.c_str(),relative_to_.c_str(),&r,&c_err);
  result->assign(r);
  rs_cstring_free(r);
  if (c_err != nullptr) {
    err->assign(c_err);
    rs_cstring_free(c_err);
  }
  return b;

  // // cannon path first
  // // TODO, fix this ...0,0
  // char* r = rs_canonicalize_path3(input.c_str(),0,0);
  // // then convert to an abs path
  // string partially_fixed = r;
  // rs_cstring_free(r);
  // string abs_input = "";

  // rs_abs_path2(partially_fixed.c_str(), &abs_input, write_to_cpp_string);
  // if (abs_input.empty()) {
  //   // If Rust failed to return a string, we consider it an error.
  //   if (err)
  //     *err = "Rustinstein: Failed to normalize path: " + input;
  //   return false;
  // }
  // // samedrive check
  // if(!rs_is_same_lexical_drive(abs_input.c_str(),relative_to_.c_str())) {
  //   *result = partially_fixed;
  //   return true;
  // }
  // *result = Relativize(abs_input, split_relative_to_, err);
  // if (!err->empty())
  //   return false;
  // // TODO: decide whether or not to keep abs paths
  // return true;
}
