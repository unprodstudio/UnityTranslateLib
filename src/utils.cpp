#include <vector>
#include <sstream>
#include <iostream>

using namespace std;

// https://stackoverflow.com/a/46931770
vector<string> split(string s, string delimiter) {
    size_t pos_start = 0, pos_end, delim_len = delimiter.length();
    vector<string> res;

    while ((pos_end = s.find(delimiter, pos_start)) != string::npos) {
        string token = s.substr(pos_start, pos_end - pos_start);
        pos_start = pos_end + delim_len;
        res.push_back(token);
    }

    res.push_back(s.substr(pos_start));
    return res;
}