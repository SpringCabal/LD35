uniform sampler2D tex0;
uniform int screenY;

const vec2 kernel = vec2(0.6, 0.7);

void main(void) {
    vec2 texCoord = vec2(gl_TextureMatrix[0] * gl_TexCoord[0]);
    gl_FragColor = vec4(0.0);
    float pixelsize = 1.0 / float(screenY);
    gl_FragColor += kernel[0] * texture2D(tex0, vec2(texCoord.s, texCoord.t + 2.0 * pixelsize));
    gl_FragColor += kernel[1] * texture2D(tex0, vec2(texCoord.s, texCoord.t + pixelsize));
    gl_FragColor += texture2D(tex0, texCoord);
    gl_FragColor += kernel[1] * texture2D(tex0, vec2(texCoord.s, texCoord.t - 1.0 * pixelsize));
    gl_FragColor += kernel[0] * texture2D(tex0, vec2(texCoord.s, texCoord.t - 2.0 * pixelsize));
}
