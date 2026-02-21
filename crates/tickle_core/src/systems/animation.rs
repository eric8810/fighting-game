use crate::math::LogicVec2;
use serde::{Deserialize, Serialize};

/// A single frame in a sprite animation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpriteFrame {
    pub atlas_index: u32,
    pub offset: LogicVec2,
}

/// Sprite animation component.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpriteAnimation {
    pub current_frame: u32,
    pub frame_duration: u32,
    pub frame_timer: u32,
    pub frames: Vec<SpriteFrame>,
    pub looping: bool,
    pub finished: bool,
}

impl SpriteAnimation {
    pub fn new(frames: Vec<SpriteFrame>, frame_duration: u32, looping: bool) -> Self {
        Self {
            current_frame: 0,
            frame_duration,
            frame_timer: 0,
            frames,
            looping,
            finished: false,
        }
    }

    pub fn current_sprite_frame(&self) -> Option<&SpriteFrame> {
        self.frames.get(self.current_frame as usize)
    }

    pub fn reset(&mut self) {
        self.current_frame = 0;
        self.frame_timer = 0;
        self.finished = false;
    }
}

/// Advances animation frames for all provided animations.
/// Handles frame_timer progression, frame advancement, looping, and one-shot completion.
pub fn animation_system(animations: &mut [SpriteAnimation]) {
    for anim in animations.iter_mut() {
        if anim.finished || anim.frames.is_empty() {
            continue;
        }

        anim.frame_timer += 1;

        if anim.frame_timer >= anim.frame_duration {
            anim.frame_timer = 0;
            anim.current_frame += 1;

            if anim.current_frame >= anim.frames.len() as u32 {
                if anim.looping {
                    anim.current_frame = 0;
                } else {
                    anim.current_frame = anim.frames.len() as u32 - 1;
                    anim.finished = true;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::LogicVec2;

    fn make_frames(count: u32) -> Vec<SpriteFrame> {
        (0..count)
            .map(|i| SpriteFrame {
                atlas_index: i,
                offset: LogicVec2::ZERO,
            })
            .collect()
    }

    #[test]
    fn test_animation_advances_timer() {
        let mut anims = [SpriteAnimation::new(make_frames(3), 4, true)];
        animation_system(&mut anims);
        assert_eq!(anims[0].frame_timer, 1);
        assert_eq!(anims[0].current_frame, 0);
    }

    #[test]
    fn test_animation_advances_frame() {
        let mut anims = [SpriteAnimation::new(make_frames(3), 2, true)];
        // Tick twice to reach frame_duration
        animation_system(&mut anims);
        animation_system(&mut anims);
        assert_eq!(anims[0].current_frame, 1);
        assert_eq!(anims[0].frame_timer, 0);
    }

    #[test]
    fn test_animation_loops() {
        let mut anims = [SpriteAnimation::new(make_frames(2), 1, true)];
        // frame 0 -> tick -> frame 1 -> tick -> frame 0 (loop)
        animation_system(&mut anims); // timer=1 -> advance to frame 1
        assert_eq!(anims[0].current_frame, 1);
        animation_system(&mut anims); // timer=1 -> advance to frame 2 -> wraps to 0
        assert_eq!(anims[0].current_frame, 0);
        assert!(!anims[0].finished);
    }

    #[test]
    fn test_animation_one_shot_finishes() {
        let mut anims = [SpriteAnimation::new(make_frames(2), 1, false)];
        animation_system(&mut anims); // -> frame 1
        assert_eq!(anims[0].current_frame, 1);
        assert!(!anims[0].finished);
        animation_system(&mut anims); // -> past end, clamp to last frame, finished
        assert_eq!(anims[0].current_frame, 1);
        assert!(anims[0].finished);
    }

    #[test]
    fn test_animation_finished_no_advance() {
        let mut anims = [SpriteAnimation::new(make_frames(2), 1, false)];
        anims[0].finished = true;
        anims[0].current_frame = 1;
        animation_system(&mut anims);
        assert_eq!(anims[0].current_frame, 1);
        assert_eq!(anims[0].frame_timer, 0);
    }

    #[test]
    fn test_animation_empty_frames() {
        let mut anims = [SpriteAnimation::new(vec![], 1, true)];
        animation_system(&mut anims);
        assert_eq!(anims[0].frame_timer, 0);
    }

    #[test]
    fn test_animation_reset() {
        let mut anim = SpriteAnimation::new(make_frames(3), 2, false);
        anim.current_frame = 2;
        anim.frame_timer = 1;
        anim.finished = true;
        anim.reset();
        assert_eq!(anim.current_frame, 0);
        assert_eq!(anim.frame_timer, 0);
        assert!(!anim.finished);
    }

    #[test]
    fn test_current_sprite_frame() {
        let anim = SpriteAnimation::new(make_frames(3), 2, true);
        let frame = anim.current_sprite_frame().unwrap();
        assert_eq!(frame.atlas_index, 0);
    }
}
