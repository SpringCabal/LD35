// Ported from the depth shader in gfx_highlight.lua.
//
// Unlike the cartoon outline this game's Flove sibling uses, the lever
// highlight deliberately keeps the fragments that fail the depth test -- the
// authored widget has that `discard` commented out -- so the glow reads
// through the lever's own geometry rather than as a rim around it.  The colour
// is hard-coded there too, so gl_Color plays no part.
uniform sampler2D tex0;
uniform int useEqualityTest;
uniform vec2 screenXY;

void main(void) {
    vec2 texCoord = vec2(gl_FragCoord.x / screenXY.x, gl_FragCoord.y / screenXY.y);
    float depth = texture2D(tex0, texCoord).z;
    if (depth < gl_FragCoord.z) {
        // discard;
    }
    gl_FragColor = vec4(0.5, 0.5, 0.2, 0.4);
}
