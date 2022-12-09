center = -24.060360
rad = 2.510002
hit = -26.569100
gravity = -0.115124

bottom = center - rad
ratio = (center - hit) / (rad)

print(f'clip len: {hit - bottom}')
print(f'bottom: {bottom}')
print(f'ratio: {ratio}')
