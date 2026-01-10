# Traffic Lights Example

## Overview

This is an example of how traffic lights could be controller using the post-haste async framework.

For this example traffic lights consists of the following components:
- Traffic lights for signalling to cars. They have 3 lights: red, amber and green. There are four valid states for the traffic lights (see appendix).
- Pedestrian lights for signalling to people who want to walk across the road. There are two lights - one that says to cross and a light that says to stop. Exactly one light should be lit at all times.
- A crossing button, which pedestrians can press to cycle the lights to a crossing state. In this example, the traffic lights will stay green until the crossing button has been pressed.

## Implementation In Post-Haste

Most of the logic is handled by the `SequencerAgent` in `sequencer.rs`. The output 'hardware' (the 3 traffic lights and 2 pedestrian lights) are controlled by the `LightsAgent` in `lights.rs`. Finally, we need to handle the input 'hardware' - the crossing button. It would be perfectly valid to model this as another agent, but the input hardware only needs to send messages - it doesn't need to recieve them. As such, in this example we will simply model it as an async task instead.

### Sequencer Agent

### Lights Agent

### Button Task

## Appendix

### Valid States For Traffic Lights

Many traffic lights in the UK will cycle through these four phases in order:
- Green: Only the green light is lit; cars can pass through.
- Green to Red: Only the amber light is lit; cars should stop if they can do so safely.
- Red: Only the red light is lit; cars should not pass. After a short time of the traffic lights being red, the pedestrians may be signalled to cross.
- Red to Green: The red and amber lights are lit; cars should prepare to start moving.