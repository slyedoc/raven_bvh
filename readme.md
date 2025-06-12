# Bevy_Slyedoc_Bvh

A Bevy Plugin for bounding volume hierarchy.

This project is just an experiment for my own enjoyment at the moment.

## Credit

This is largely based on the amazing [tutorial series](https://jacco.ompf2.com/2022/04/13/how-to-build-a-bvh-part-1-basics/) by Jacco Bikker.  Go check it out if bvh's interest you.

And of course bevy and its community.

## Context

This allows for Bvh based on meshs that can be ray cast agaist, those can in turn be ray cast agaist with a tlas that is currently built every frame.

To test how fast and and correct those ray's are, there is a "camera" feauture to visualize the rays.  This should only be used for debugging to benchmarking.  You would not use the feature for *production*

## Overview

  
## Other Resources
- [tutorial series](https://jacco.ompf2.com/2022/04/13/how-to-build-a-bvh-part-1-basics/) by Jacco Bikker.
- [Ray Tracing in One Weekend](https://raytracing.github.io/) How everyone gets started with raytracing anymore.
- [NVIDIA Raytrace Tutoral](https://developer.nvidia.com/rtx/raytracing/vkray) This is c++ and for the vulkan extention, and it was pretty rough to get though
- [Trace-Efficiency](https://www.nvidia.com/docs/IO/76976/HPG2009-Trace-Efficiency.pdf) Old NVidia paper exploring different ideas (Jacco Bikker tweet)