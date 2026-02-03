# Traffic Lights Example

## Overview

This is an example of how traffic lights could be controlled using the post-haste async framework.

For this example the traffic lights consists of the following components:
- Traffic lights for signalling to cars. They have 3 lights: red, amber and green. There are four valid states for the traffic lights:
```
1) Red              2) Red to Green     3) Green            4) Green to Red
----                ----                ----                ----
|██|                |██|                |  |                |  |
----                ----                ----                ----
|  |                |██|                |  |                |██|
----                ----                ----                ----
|  |                |  |                |██|                |  |
----                ----                ----                ----
```
- Pedestrian lights for signalling to people who want to walk across the road. There are two potential states:
```
1) Stop             2) Cross
-------             -------
|STOP |             |     |
|     |             |CROSS|
-------             -------
```
- A crossing button, which pedestrians can press to cycle the lights to a crossing state. In this example, the traffic lights will stay green until the crossing button has been pressed. The button has a light which indicates that the button has been pressed.

## Implementation In Post-Haste

There are many possible implementations; for this example there are:

- Two agents - Display and Sequencer. 
- The Display agent is responsible for displaying text in the terminal (no other part of the program will print to the terminal). In a real embedded software project, you might have an agent for controlling GPIO pins.
- The Sequencer agent is responsible for managing the states of the lights, and choosing what to do when the button is pressed. It sends delayed messages to itself to manage the timings of the lights - this adds a bit of complexity but means that all agents are always available to recieve messages.
- The button functionality could also have been implemented as an agent, but as the button only sends messages and does not send any, it can simply be a tokio task which sends post-haste messages.
