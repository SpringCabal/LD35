// The spirit-wisp billboard, ported from the shader in unit_wisp_draw.lua.
//
// Stacked translucent triangle fans were used here before; they banded into a
// visible octagon and, being tinted blue, looked nothing like the authored
// sprite.  The authored version adds dist^3 to all four channels equally, which
// is what makes the core read white.
uniform float time;
uniform float spiritAmount;
uniform float unitID;
uniform float deathAmount;

float rand(vec2 n) {
  return fract(sin(dot(n, vec2(12.9898, 4.1414))) * 43758.5453);
}

void main(void) {
  float dx = 0.5 - gl_TexCoord[0].x;
  float dy = 0.5 - gl_TexCoord[0].y;
  dy *= 1.0 - deathAmount / 3.0;
  float dist = 1.0 - 2.5 * (1.5 * dx * dx + dy * dy);

  float f = 0.0;
  vec2 coord = gl_TexCoord[0].xy;
  for (int i = -3; i <= 3; i++) {
    for (int j = -3; j <= 3; j++) {
      coord = floor(gl_TexCoord[0].xy * 50.0 + vec2(i, j) + 0.5);
      f += rand(coord * time * unitID);
    }
  }
  f = f / 49.0 * dist;
  vec4 color = vec4(f, f, f, f);
  color.r *= rand(vec2(cos(unitID), sin(unitID)));
  color.g *= rand(vec2(sin(unitID), cos(unitID)));
  color.b *= rand(vec2(-cos(unitID), -sin(unitID)));

  color += dist * dist * dist / 1.2 * (1.0 - deathAmount / 1.5);
  color *= spiritAmount;
  gl_FragColor = color;
}
