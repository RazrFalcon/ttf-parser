#include <fstream>
#include <streambuf>

#include <benchmark/benchmark.h>

#include <ft2build.h>
#include FT_FREETYPE_H
#include FT_OUTLINE_H

#include <ttfparser.h>

#define STB_TRUETYPE_IMPLEMENTATION
#include "stb_truetype.h"

#include <hb.h>

namespace FT {
struct Outliner
{
    static int moveToFn(const FT_Vector *, void *user)
    {
        auto self = static_cast<Outliner *>(user);
        self->counter += 1;
        return 0;
    }

    static int lineToFn(const FT_Vector *, void *user)
    {
        auto self = static_cast<Outliner *>(user);
        self->counter += 1;
        return 0;
    }

    static int quadToFn(const FT_Vector *, const FT_Vector *, void *user)
    {
        auto self = static_cast<Outliner *>(user);
        self->counter += 1;
        return 0;
    }

    static int cubicToFn(const FT_Vector *, const FT_Vector *, const FT_Vector *, void *user)
    {
        auto self = static_cast<Outliner *>(user);
        self->counter += 1;
        return 0;
    }

    uint32_t counter = 0;
};

class Font
{
public:
    Font(const std::string &path, const uint32_t index = 0)
    {
        if (FT_Init_FreeType(&m_library)) {
            throw "failed to init FreeType";
        }

        std::ifstream s(path);
        std::vector<char> data((std::istreambuf_iterator<char>(s)),
                                std::istreambuf_iterator<char>());
        m_fontData = std::move(data);

        if (FT_New_Memory_Face(m_library, (FT_Byte*)m_fontData.data(), m_fontData.size(), index, &m_face)) {
            throw "failed to open a font";
        }
    }

    ~Font()
    {
        if (m_face) {
            FT_Done_Face(m_face);
        }

        FT_Done_FreeType(m_library);
    }

    uint16_t numberOfGlyphs() const
    {
        return (uint16_t)m_face->num_glyphs;
    }

    uint32_t outline(const uint16_t gid) const
    {
        if (FT_Load_Glyph(m_face, gid, FT_LOAD_NO_SCALE | FT_LOAD_NO_BITMAP)) {
            throw "failed to load a glyph";
        }

        Outliner outliner;

        FT_Outline_Funcs funcs;
        funcs.move_to = outliner.moveToFn;
        funcs.line_to = outliner.lineToFn;
        funcs.conic_to = outliner.quadToFn;
        funcs.cubic_to = outliner.cubicToFn;
        funcs.shift = 0;
        funcs.delta = 0;

        if (FT_Outline_Decompose(&m_face->glyph->outline, &funcs, &outliner)) {
            throw "failed to outline a glyph";
        }

        return outliner.counter;
    }

private:
    std::vector<char> m_fontData;
    FT_Library m_library = nullptr;
    FT_Face m_face = nullptr;
};
}

namespace STB {
class Font
{
public:
    Font(const std::string &path, const uint32_t index = 0)
    {
        std::ifstream s(path);
        std::vector<char> data((std::istreambuf_iterator<char>(s)),
                                std::istreambuf_iterator<char>());
        m_fontData = std::move(data);

        if (!stbtt_InitFont(&m_font, (const uint8_t *)m_fontData.data(), 0)) {
            throw "failed to open a font";
        }
    }

    uint16_t numberOfGlyphs() const
    {
        return (uint16_t)m_font.numGlyphs;
    }

    uint32_t outline(const uint16_t gid) const
    {
        stbtt_vertex *vertices;
        const auto num_verts = stbtt_GetGlyphShape(&m_font, gid, &vertices);
        stbtt_FreeShape(&m_font, vertices);
        return num_verts;
    }

private:
    std::vector<char> m_fontData;
    stbtt_fontinfo m_font;
};
}


namespace HB {
struct Outliner
{
    static void moveToFn(hb_position_t x, hb_position_t y, void *user)
    {
        auto self = static_cast<Outliner *>(user);
        self->counter += 1;
    }

    static void lineToFn(hb_position_t x, hb_position_t y, void *user)
    {
        auto self = static_cast<Outliner *>(user);
        self->counter += 1;
    }

    static void quadToFn(hb_position_t c_x, hb_position_t c_y, hb_position_t x, hb_position_t y, void *user)
    {
        auto self = static_cast<Outliner *>(user);
        self->counter += 1;
    }

    static void cubicToFn(hb_position_t c1_x, hb_position_t c1_y, hb_position_t c2_x, hb_position_t c2_y, hb_position_t x, hb_position_t y, void *user)
    {
        auto self = static_cast<Outliner *>(user);
        self->counter += 1;
    }

    uint32_t counter = 0;
};

class Font
{
public:
    Font(const std::string &path, const uint32_t index = 0)
    {
        std::ifstream s(path);
        std::vector<char> data((std::istreambuf_iterator<char>(s)),
                                std::istreambuf_iterator<char>());
        m_fontData = std::move(data);

        funcs = hb_draw_funcs_create();
        hb_draw_funcs_set_move_to_func(funcs, (hb_draw_move_to_func_t) Outliner::moveToFn);
        hb_draw_funcs_set_line_to_func(funcs, (hb_draw_line_to_func_t) Outliner::lineToFn);
        hb_draw_funcs_set_quadratic_to_func(funcs, (hb_draw_quadratic_to_func_t) Outliner::quadToFn);
        hb_draw_funcs_set_cubic_to_func(funcs, (hb_draw_cubic_to_func_t) Outliner::cubicToFn);
        hb_blob_t *blob = hb_blob_create((const char *)m_fontData.data (), m_fontData.size (), HB_MEMORY_MODE_WRITABLE, nullptr, nullptr);
        hb_face_t *face = hb_face_create(blob, 0);
        hb_blob_destroy(blob);
        font = hb_font_create (face);
        hb_face_destroy(face);
    }

    uint16_t numberOfGlyphs() const
    {
        return hb_face_get_glyph_count(hb_font_get_face(font));
    }

    uint32_t outline(const uint16_t gid) const
    {
        Outliner outliner = {0};
        hb_font_draw_glyph(font, gid, funcs, &outliner);
        return outliner.counter;
    }

    ~Font()
    {
        hb_font_destroy(font);
        hb_draw_funcs_destroy(funcs);
    }

private:
    hb_font_t *font;
    std::vector<char> m_fontData;
    hb_draw_funcs_t *funcs;
};
}

namespace TTFP {
struct Outliner
{
    static void moveToFn(float x, float y, void *user)
    {
        auto self = static_cast<Outliner *>(user);
        self->counter += 1;
    }

    static void lineToFn(float x, float y, void *user)
    {
        auto self = static_cast<Outliner *>(user);
        self->counter += 1;
    }

    static void quadToFn(float x1, float y1, float x, float y, void *user)
    {
        auto self = static_cast<Outliner *>(user);
        self->counter += 1;
    }

    static void curveToFn(float x1, float y1, float x2, float y2, float x, float y, void *user)
    {
        auto self = static_cast<Outliner *>(user);
        self->counter += 1;
    }

    static void closePathFn(void *user)
    {
        auto self = static_cast<Outliner *>(user);
        self->counter += 1;
    }

    uint32_t counter = 0;
};

class Font
{
public:
    Font(const std::string &path, const uint32_t index = 0)
    {
        std::ifstream s(path);
        std::vector<char> data((std::istreambuf_iterator<char>(s)),
                                std::istreambuf_iterator<char>());
        m_fontData = std::move(data);

        m_font = ttfp_create_font((const uint8_t *)m_fontData.data(), m_fontData.size(), index);
        if (!m_font) {
            throw "failed to parse a font";
        }
    }

    ~Font()
    {
        if (m_font) {
            ttfp_destroy_font(m_font);
        }
    }

    uint16_t numberOfGlyphs() const
    {
        return ttfp_get_number_of_glyphs(m_font);
    }

    uint32_t outline(const uint16_t gid) const
    {
        Outliner outliner;

        ttfp_outline_builder builder;
        builder.move_to = outliner.moveToFn;
        builder.line_to = outliner.lineToFn;
        builder.quad_to = outliner.quadToFn;
        builder.curve_to = outliner.curveToFn;
        builder.close_path = outliner.closePathFn;

        ttfp_bbox bbox;
        ttfp_outline_glyph(m_font, builder, &outliner, gid, &bbox);

        return outliner.counter;
    }

private:
    std::vector<char> m_fontData;
    ttfp_font *m_font = nullptr;
};
}

static void freetype_outline_glyf(benchmark::State &state)
{
    FT::Font font("../fonts/SourceSansPro-Regular.ttf", 0);
    for (auto _ : state) {
        for (uint i = 0; i < font.numberOfGlyphs(); i++) {
            font.outline(i);
        }
    }
}
BENCHMARK(freetype_outline_glyf);

static void freetype_outline_cff(benchmark::State &state)
{
    FT::Font font("../fonts/SourceSansPro-Regular.otf", 0);
    for (auto _ : state) {
        for (uint i = 0; i < font.numberOfGlyphs(); i++) {
            font.outline(i);
        }
    }
}
BENCHMARK(freetype_outline_cff);

static void stb_truetype_outline_glyf(benchmark::State &state)
{
    STB::Font font("../fonts/SourceSansPro-Regular.ttf", 0);
    const auto numberOfGlyphs = font.numberOfGlyphs();
    for (auto _ : state) {
        for (uint i = 0; i < numberOfGlyphs; i++) {
            font.outline(i);
        }
    }
}
BENCHMARK(stb_truetype_outline_glyf);

static void stb_truetype_outline_cff(benchmark::State &state)
{
    STB::Font font("../fonts/SourceSansPro-Regular.otf", 0);
    const auto numberOfGlyphs = font.numberOfGlyphs();
    for (auto _ : state) {
        for (uint i = 0; i < numberOfGlyphs; i++) {
            font.outline(i);
        }
    }
}
BENCHMARK(stb_truetype_outline_cff);

static void ttf_parser_outline_glyf(benchmark::State &state)
{
    TTFP::Font font("../fonts/SourceSansPro-Regular.ttf", 0);
    const auto numberOfGlyphs = font.numberOfGlyphs();
    for (auto _ : state) {
        for (uint i = 0; i < numberOfGlyphs; i++) {
            font.outline(i);
        }
    }
}
BENCHMARK(ttf_parser_outline_glyf);

static void ttf_parser_outline_cff(benchmark::State &state)
{
    TTFP::Font font("../fonts/SourceSansPro-Regular.otf", 0);
    const auto numberOfGlyphs = font.numberOfGlyphs();
    for (auto _ : state) {
        for (uint i = 0; i < numberOfGlyphs; i++) {
            font.outline(i);
        }
    }
}
BENCHMARK(ttf_parser_outline_cff);

static void harfbuzz_outline_glyf(benchmark::State &state)
{
    HB::Font font("../fonts/SourceSansPro-Regular.ttf", 0);
    for (auto _ : state) {
        for (uint i = 0; i < font.numberOfGlyphs(); i++) {
            font.outline(i);
        }
    }
}
BENCHMARK(harfbuzz_outline_glyf);

static void harfbuzz_outline_cff(benchmark::State &state)
{
    HB::Font font("../fonts/SourceSansPro-Regular.otf", 0);
    for (auto _ : state) {
        for (uint i = 0; i < font.numberOfGlyphs(); i++) {
            font.outline(i);
        }
    }
}
BENCHMARK(harfbuzz_outline_cff);

BENCHMARK_MAIN();
