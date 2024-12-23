#!python3
from PIL import Image
image = Image.open('./input.png')
image = image.convert('L')
image = image.quantize(colors=256, method=2)
image.save('./input-out.png')
