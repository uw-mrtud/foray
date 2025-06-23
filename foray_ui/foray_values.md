# Problem summary

Researchers want to develop new methods for transforming raw data from sensors (medical imaging devices specifically), into useful ouptuts (mostly images of the inside of humans)

Traditionally, this is done using general purpose scientific computation tools like matlab, python (numpy/scipy), and bespoke C implementations when there are performance bottlenecks.

## Example User 

Jim is frustrated with the current methods, because these tools require researchers to work in an abstraction that is far removed from their problem domain. The researchers have to shape their problem into textual programs, creating a large amount of friction when exploring new ideas


Jim is interested in developing a tool that is designed specifically for this domain to enable researchers to work more intuitively in the problem space.

By fully embracing human centered desing principles, he hopes that this tool can make his research less limited by irrelevant abstractions, and better suited to the human minds exploring new ideas.


# Constraints

1. Usable version (for daily users) within ~1 year of development
2. User extendable for scientific computation
    - Basically this means that I have something that is on-par with numpy/scipy, or let users integrate nicely with numpy/scipy or just all of python



# Mechanics


1. Easy/Default visualization
2. Quick/immediate idea exploration
    - the loop from ideation to implementation to observation
    1. Ideation
        This is your reponsibility, it happens in your  brain.
    2. Implementation 
        This is a side effect of your brain. We need to caputer it and store it in the software.
        This process should feel invisible, like there isn't really a wrong way to do it. 
        For this to happen it would be great if your mental picture of what to enter into the computer was the same as what you enter into the computer.

        i.e. I want to take the fourier transform of this signal, encode phase as a color, and plot the result.

        That shouldn't be more than 3 actions, and it should be very difficult to make a mistake.
        In python, there's a ton of ways that you could make a mistake, and there are an infinite possible errors that could be introduced with an errant keystroke. 

    3. 


        

    3. 


    - should be seamless. No thinking about what matplot lib params need to be set to make sense of a new piece of data

    - Fast, Continuous paramter scrubbing
        - The difficulty of trying out a new idea is a tradgedy.
        - If you want to know what would happen if you increased a parameter, you should never be more than a couple discrete interactions away from knowing what turning that knob will do.
        - Immediate feedback is essential for as many tasks as possible.
        - how many values would users try when editing a python
    - Clone data, or algorithms and set them aside for a second.
        try a new change and see how it goes. Make it simple to revert back, or solidify the fork, or clone again (make it easy to paramaterize the clone, so that a larger/continous space can be explored.)

``` Fork
  _
 /
-
 \_
```

``` Revert
  _
 /
-
```

``` Apply
-
 \_
```

``` parameterize
 /--
 /--
 /--
-
 \__
 \__
 \__
 \__
```
