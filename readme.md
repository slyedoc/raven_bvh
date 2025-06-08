# Bevy_Slyedoc_Bvh

A Bevy Plugin for bounding volume hierarchy.

This project is just an experiment for my own enjoyment at the moment.

## Credit

This is largely based on the amazing [tutorial series](https://jacco.ompf2.com/2022/04/13/how-to-build-a-bvh-part-1-basics/) by Jacco Bikker.  Go check it out if bvh's interest you.

And of course bevy and its community.

## Context

Any *production* bevy raytracing solution would most likely be based on [VK_KHR_ray_tracing_pipeline](https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VK_KHR_ray_tracing_pipeline.html) assuming you want to take advantage of the latest GPU hardware.  That would require deep knowledge of vulkan, wgpu, and bevy.  This is not that project, not yet at least.

How does that hardware help, in short:

> RTX cards feature fixed-function "RT cores" that are designed to accelerate mathematical operations needed to simulate rays, such as bounding volume hierarchy traversal.  - [wikipedia](https://en.wikipedia.org/wiki/Nvidia_RTX#Ray_tracing)

These cores speed up the BVH traversal, but the acceleration structure is computed on the cpu.  This project is an experiment on building the bvh natively in bevy.  Of coarse if you have a bvh, you are going to try raytracing with it. =-)

## Overview

This is broke up into 2 crates:
- bvh: plugin produces a bvh bevy resource that can be used from any system.
  - Has a cpu based camera for stress testing and debugging
  
## Notes

Currently, we are duplicating mesh data at the moment and rebuilding the bvh each frame. Will add refit back sometime.
## Other Resources

- [Ray Tracing in One Weekend](https://raytracing.github.io/) How everyone gets started with raytracing anymore.
- [NVIDIA Raytrace Tutoral](https://developer.nvidia.com/rtx/raytracing/vkray) This is c++ and for the vulkan extention, and it was pretty rough to get though
- [Trace-Efficiency](https://www.nvidia.com/docs/IO/76976/HPG2009-Trace-Efficiency.pdf) Old NVidia paper exploring different ideas (Jacco Bikker tweet)