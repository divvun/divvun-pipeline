#include "wrapper.hpp"

#include <istream>
#include <iostream>

class membuf : public std::basic_streambuf<char>
{
public:
    membuf(const uint8_t *p, size_t l)
    {
        setg((char *)p, (char *)p, (char *)p + l);
    }
};

class memstream : public std::istream
{
public:
    memstream(const uint8_t *p, size_t l) : std::istream(&_buffer),
                                            _buffer(p, l)
    {
        rdbuf(&_buffer);
    }

private:
    membuf _buffer;
};

extern "C" std::stringstream *hfst_run(hfst_ol_tokenize::TokenizeSettings *settings, const uint8_t *pmatch_data, size_t pmatch_size, const uint8_t *input_data, size_t input_size, size_t *output_size)
{
    memstream pmatch_stream(pmatch_data, pmatch_size);
    hfst_ol::PmatchContainer container(pmatch_stream);

    memstream input_stream(input_data, input_size);

    auto output = new std::stringstream(std::ios::in | std::ios::out | std::ios::binary);
    hfst_ol_tokenize::process_input(container, input_stream, *output, *settings);
    output->seekg(0, output->end);
    *output_size = output->tellg();

    return output;
}

extern "C" void hfst_free(std::stringstream *stream)
{
    delete stream;
}

extern "C" void hfst_copy_output(std::stringstream *stream, char *output, size_t size)
{
    stream->seekg(0, stream->beg);
    stream->read(output, size);
}
