#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "ttfparser.h"

void move_to_cb(float x, float y, void *data)
{
    uint32_t *counter = (uint32_t*)data;
    *counter += 1;
}

void line_to_cb(float x, float y, void *data)
{
    uint32_t *counter = (uint32_t*)data;
    *counter += 1;
}

void quad_to_cb(float x1, float y1, float x, float y, void *data)
{
    uint32_t *counter = (uint32_t*)data;
    *counter += 1;
}

void curve_to_cb(float x1, float y1, float x2, float y2, float x, float y, void *data)
{
    uint32_t *counter = (uint32_t*)data;
    *counter += 1;
}

void close_path_cb(void *data)
{
    uint32_t *counter = (uint32_t*)data;
    *counter += 1;
}

int main() {
    // Read the file first.
    FILE *file = fopen("../benches/fonts/SourceSansPro-Regular.ttf", "rb");
    if (file == NULL) {
        return -1;
    }

    fseek(file, 0, SEEK_END);
    long fsize = ftell(file);
    fseek(file, 0, SEEK_SET);

    char *font_data = malloc(fsize + 1);
    fread(font_data, 1, fsize, file);
    fclose(file);

    // Test functions.
    // We mainly interested in linking errors.
    assert(ttfp_fonts_in_collection(font_data, fsize) == -1);

    ttfp_font *font = ttfp_create_font(font_data, fsize, 0);
    assert(font);

    assert(ttfp_has_table(font, TTFP_TABLE_NAME_HEADER));
    assert(!ttfp_has_table(font, TTFP_TABLE_NAME_VERTICAL_ORIGIN));

    uint16_t a_gid = ttfp_get_glyph_index(font, 0x0041); // A
    assert(a_gid == 2);
    assert(ttfp_get_glyph_index(font, 0xFFFFFFFF) == 0);
    assert(ttfp_get_glyph_var_index(font, 0x0041, 0xFE03) == 0);

    assert(ttfp_get_glyph_hor_advance(font, 0x0041) == 544);
    assert(ttfp_get_glyph_hor_side_bearing(font, 0x0041) == 3);
    assert(ttfp_get_glyph_ver_advance(font, 0x0041) == 0);
    assert(ttfp_get_glyph_ver_side_bearing(font, 0x0041) == 0);
    assert(ttfp_get_glyph_y_origin(font, a_gid) == 0);

    assert(ttfp_get_name_records_count(font) == 20);
    ttfp_name_record record;
    assert(ttfp_get_name_record(font, 100, &record) == false);
    assert(ttfp_get_name_record(font, 1, &record) == true);
    assert(record.name_id == 1);

    char family_name[30];
    assert(ttfp_get_name_record_string(font, 1, family_name, 30));

    assert(ttfp_get_units_per_em(font) == 1000);
    assert(ttfp_get_ascender(font) == 984);
    assert(ttfp_get_descender(font) == -273);
    assert(ttfp_get_height(font) == 1257);
    assert(ttfp_get_line_gap(font) == 0);
    assert(ttfp_is_regular(font) == true);
    assert(ttfp_is_italic(font) == false);
    assert(ttfp_is_bold(font) == false);
    assert(ttfp_is_oblique(font) == false);
    assert(ttfp_get_weight(font) == 400);
    assert(ttfp_get_width(font) == 5);
    assert(ttfp_get_x_height(font) == 486);
    assert(ttfp_get_number_of_glyphs(font) == 1974);

    ttfp_line_metrics line_metrics;
    assert(ttfp_get_underline_metrics(font, &line_metrics));
    assert(line_metrics.position == -50);
    assert(line_metrics.thickness == 50);

    assert(ttfp_get_strikeout_metrics(font, &line_metrics));
    assert(line_metrics.position == 291);
    assert(line_metrics.thickness == 50);

    ttfp_script_metrics script_metrics;
    assert(ttfp_get_subscript_metrics(font, &script_metrics));
    assert(script_metrics.x_size == 650);
    assert(script_metrics.y_size == 600);
    assert(script_metrics.x_offset == 0);
    assert(script_metrics.y_offset == 75);

    assert(ttfp_get_superscript_metrics(font, &script_metrics));
    assert(script_metrics.x_size == 650);
    assert(script_metrics.y_size == 600);
    assert(script_metrics.x_offset == 0);
    assert(script_metrics.y_offset == 350);

    assert(ttfp_get_glyph_class(font, a_gid) == 1);
    assert(ttfp_get_glyph_mark_attachment_class(font, a_gid) == 0);
    assert(ttfp_is_mark_glyph(font, a_gid) == false);

    ttfp_rect a_bbox = {0};
    assert(ttfp_get_glyph_bbox(font, a_gid, &a_bbox));
    assert(a_bbox.x_min == 3);
    assert(a_bbox.y_min == 0);
    assert(a_bbox.x_max == 541);
    assert(a_bbox.y_max == 656);

    assert(!ttfp_get_glyph_bbox(font, 0xFFFF, &a_bbox));

    uint32_t counter = 0;
    ttfp_outline_builder builder;
    builder.move_to = move_to_cb;
    builder.line_to = line_to_cb;
    builder.quad_to = quad_to_cb;
    builder.curve_to = curve_to_cb;
    builder.close_path = close_path_cb;
    assert(ttfp_outline_glyph(font, builder, &counter, a_gid, &a_bbox));
    assert(counter == 20);
    // The same as via ttfp_get_glyph_bbox()
    assert(a_bbox.x_min == 3);
    assert(a_bbox.y_min == 0);
    assert(a_bbox.x_max == 541);
    assert(a_bbox.y_max == 656);

    char glyph_name[256];
    assert(ttfp_get_glyph_name(font, a_gid, glyph_name));
    assert(strcmp(glyph_name, "A") == 0);

    ttfp_destroy_font(font);

    return 0;
}
