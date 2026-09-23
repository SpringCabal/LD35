// The combine pass from gfx_eyes_effect.lua: the bloom composite, the spirit
// colour grade, the vignette and the grain, in the authored order.
//
// texture1 is the blurred bright-pass: the wisp's halo and every highlight's
// bloom.
uniform sampler2D texture0;
uniform sampler2D texture1;
uniform float illuminationThreshold;
uniform float fragMaxBrightness;
uniform int debugDraw;

uniform float time;
uniform float spiritAmount;
uniform float noiseAmount;
uniform float gameOverAmount;

float rand(vec2 co) {
  return fract(sin(dot(co.xy, vec2(12.9898, 78.233))) * 43758.5453);
}

void main(void) {
  vec2 C0 = vec2(gl_TexCoord[0]);
  vec4 S0 = texture2D(texture0, C0);
  vec4 S1 = texture2D(texture1, C0);
  S1 = vec4(S1.rgb * fragMaxBrightness / max(1.0 - illuminationThreshold, 0.0001), 1.0);

  gl_FragColor = bool(debugDraw) ? S1 : S0 + S1;

  gl_FragColor.rg *= 0.4 + 0.6 * (1.0 - spiritAmount);
  gl_FragColor.b += 0.1 - 0.1 * (1.0 - spiritAmount);
  gl_FragColor.b *= 1.2 - 0.2 * (1.0 - spiritAmount);

  float dtime1 = sin(2.0 * time) / 3.14 / 4.0;
  float dx = 0.5 - C0.x;
  float dy = 0.5 - C0.y;
  float dist = 1.0 - 2.5 * (1.5 * dx * dx + dy * dy);

  if (spiritAmount == 2.0) {
    gl_FragColor = vec4(0.2, 0.2, 0.2, 1.0);
  }

  gl_FragColor.rgb *= dist;
  // Grain.  Feeding gl_FragCoord into this sin() hash instead loses float
  // precision at screen scale and turns the noise into coarse diagonal
  // banding; the authored version keeps both hash inputs small.
  gl_FragColor.rgb += rand(vec2(dtime1, C0.x + 10.0 * C0.y)) / 12.0 * noiseAmount;

  gl_FragColor.rgba += 0.5 * gameOverAmount;
  gl_FragColor.rgba *= 1.0 + gameOverAmount;
}
