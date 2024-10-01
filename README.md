# Pico Snake Game
A game in rust that resembles the classic Snake Game from the Nokia 5110 phones era.

## Description

This projects resembles the classic Snake Game from the Nokia phones, having the same rules and mechanics. You control the Snake using a 2-axis joystick. Your score is updated live on a 4 digit display (maximum score is 9999, then it resets). For every interaction with "food", a buzzer will play a sound. If you eat yourself, it's game over and you have to start again. You can also pass through walls (surely a choice). The game itself is displayed on an actual Nokia 5110 LCD-like display. Good luck and enjoy!

## Architecture 

![architecture](assets/hardware/Architecture.png)

## Hardware
### Pictures
<table>
<tr>
<td>
  
![opened](assets/hardware/Opened.jpg)

</td>
<td>
  
![game](assets/hardware/Game.jpg)

</td>
</tr>
<tr>
<td>
  
![gameover](assets/hardware/GameOver.jpg)

  </td>
<td>
  
![sprofile](assets/hardware/SideProfile.jpg)

</td>
</tr>
</table>

### Usage of components

- **Joystick Module** is used for controlling the snake (interacting with the game) and it's connected to ADC pins such that the analog voltage is converted to digital values.
- **Passive Buzzer** is used for *beeping* in various situations (sound feedback from the game).
- **Nokia 5110 Display** is used for displaying the game itself.
- **TM1637 Module** is used for keeping track of the score while playing.
- **Raspberry Pi Pico H** is used as the main component (microcontroller) responsible for processing input data.
- **Breadboard** is used for connecting every pin of the hardware components.

### Schematics

![kicadschematic](assets/kicad/Schematic.svg)
