import keyboard
import random
import pygame

# confetti colors
colors = [(255, 0, 0), (0, 255, 0), (0, 0, 255), (255, 255, 0), (255, 0, 255), (0, 255, 255)]

# confetti properties
confetti = [{"x": 
             random.randint(0, 800), 
             "y": random.randint(0,600), 
             "color": random.choice(colors), 
             "size": random.randint(5, 15), 
             "speed": random.uniform(1, 3)} for _ in range(100)]


pygame.init()
screen = pygame.display.set_mode((800, 600), pygame.NOFRAME, pygame.SRCALPHA)

def draw_confetti():
    screen.fill((0, 0, 0, 0))  # Clear screen with transparent background
    for piece in confetti:
        pygame.draw.rect(screen, piece["color"], (piece["x"], piece["y"], piece["size"], piece["size"]))
        piece["y"] += piece["speed"]
        if piece["y"] > 600:
            piece["y"] = 0
            piece["x"] = random.randint(0, 800)
            piece["color"] = random.choice(colors)
            piece["size"] = random.randint(5, 15)
            piece["speed"] = random.uniform(1, 3)

def confetti_mainloop():
    running = True
    clock = pygame.time.Clock()
    while running:
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                running = False
        draw_confetti()
        pygame.display.flip()
        clock.tick(30)
        
def on_hotkey():
    print("Hotkey activated!")
    confetti_mainloop()


keyboard.add_hotkey('ctrl+shift+c', on_hotkey)
keyboard.wait('esc')  # Keep the program running until 'esc' is pressed

