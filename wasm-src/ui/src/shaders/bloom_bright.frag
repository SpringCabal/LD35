// The bloom bright-pass from gfx_eyes_effect.lua: keep only what is brighter
// than the illumination threshold, and keep only the excess.
uniform sampler2D texture0;
uniform float illuminationThreshold;
uniform float inverseRX;
uniform float inverseRY;

void main(void) {
    vec2 C0 = vec2(gl_TexCoord[0]);
    vec3 color = vec3(texture2D(texture0, C0));
    float illum = dot(color, vec3(0.2990, 0.5870, 0.1140));
    if (illum > illuminationThreshold) {
        gl_FragColor = vec4((color - color * (illuminationThreshold / max(illum, 0.00001))), 1.0);
    } else {
        gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0);
    }
}
