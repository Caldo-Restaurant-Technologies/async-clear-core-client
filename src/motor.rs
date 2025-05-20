use std::time::Duration;

use anyhow::{Result, anyhow};
use serde::Serialize;
use tokio::sync::mpsc::Sender;
use tokio::time::MissedTickBehavior;

use crate::controller::{Message, check_reply, make_prefix};
use crate::send_recv::SendRecv;
use crate::{ascii_to_int, num_to_bytes};



#[derive(Clone)]
pub struct MotorBuilder {
    pub id: usize,
    pub scale: usize,
}

#[derive(Debug, PartialOrd, PartialEq, Serialize)]
pub enum Status {
    Disabled,
    Enabling,
    Faulted,
    Ready,
    Moving,
}

#[derive(Clone)]
pub struct ClearCoreMotor {
    pub id: u8,
    prefix: [u8; 3],
    scale: usize,
    drive_sender: Sender<Message>,
}

impl SendRecv for ClearCoreMotor {
    fn get_sender(&self) -> &Sender<Message> {
        &self.drive_sender
    }
}

impl ClearCoreMotor {
    pub fn new(id: usize, scale: usize, drive_sender: Sender<Message>) -> Self {
        let id = id as u8;
        let prefix = make_prefix(b'M', id);
        ClearCoreMotor {
            id,
            prefix,
            scale,
            drive_sender,
        }
    }

    pub async fn enable(&self) -> Result<()> {
        let enable_cmd = [2, b'M', self.id + 48, b'E', b'N', 13];
        let resp = self.write(enable_cmd.as_ref()).await;
        check_reply(&resp)?;
        let mut tick_interval = tokio::time::interval(Duration::from_millis(250));
        tick_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        while self.get_status().await? == Status::Enabling {
            tick_interval.tick().await;
        }
        if self.get_status().await? == Status::Faulted {
            Err(anyhow!("motor faulted".to_string()))
        } else {
            Ok(())
        }
    }

    pub async fn disable(&self) -> Result<()> {
        let enable_cmd = [2, b'M', self.id + 48, b'D', b'E', 13];
        let resp = self.write(enable_cmd.as_ref()).await;
        check_reply(resp.as_ref())?;
        Ok(())
    }

    pub async fn absolute_move(&self, position: f64) -> Result<()> {
        let position = num_to_bytes((position * (self.scale as f64)).trunc() as isize);
        let mut msg: Vec<u8> = Vec::with_capacity(position.len() + self.prefix.len() + 1);
        msg.extend_from_slice(self.prefix.as_slice());
        msg.extend_from_slice(b"AM");
        msg.extend_from_slice(position.as_slice());
        msg.push(13);
        let resp = self.write(msg.as_slice()).await;
        check_reply(&resp)?;
        Ok(())
    }

    pub async fn relative_move(&self, position: f64) -> Result<()> {
        let position = num_to_bytes((position * (self.scale as f64)).trunc() as isize);
        let mut msg: Vec<u8> = Vec::with_capacity(position.len() + self.prefix.len() + 1);
        msg.extend_from_slice(self.prefix.as_slice());
        msg.extend_from_slice(b"RM");
        msg.extend_from_slice(position.as_slice());
        msg.push(13);
        let resp = self.write(msg.as_slice()).await;
        check_reply(&resp)?;
        Ok(())
    }

    pub async fn jog(&self, speed: f64) -> Result<()> {
        let speed = num_to_bytes((speed * (self.scale as f64)).trunc() as isize);
        let mut msg: Vec<u8> = Vec::with_capacity(speed.len() + self.prefix.len() + 1);
        msg.extend_from_slice(self.prefix.as_slice());
        msg.extend_from_slice(b"JG");
        msg.extend_from_slice(speed.as_slice());
        msg.push(13);
        let resp = self.write(msg.as_slice()).await;
        check_reply(&resp)?;
        Ok(())
    }

    pub async fn abrupt_stop(&self) -> Result<()> {
        let stop_cmd = [2, b'M', self.id + 48, b'A', b'S', 13];
        let resp = self.write(stop_cmd.as_ref()).await;
        check_reply(&resp)?;
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let stop_cmd = [2, b'M', self.id + 48, b'S', b'T', 13];
        let resp = self.write(stop_cmd.as_ref()).await;
        check_reply(&resp)?;
        Ok(())
    }

    pub async fn set_position(&self, position: isize) -> Result<()> {
        let pos = num_to_bytes(position * self.scale as isize);
        let mut msg: Vec<u8> = Vec::with_capacity(pos.len() + self.prefix.len() + 1);
        msg.extend_from_slice(self.prefix.as_slice());
        msg.extend_from_slice(b"SP");
        msg.extend_from_slice(pos.as_slice());
        msg.push(13);
        let resp = self.write(msg.as_slice()).await;
        check_reply(&resp)?;
        Ok(())
    }

    pub async fn set_velocity(&self, mut velocity: f64) -> Result<()> {
        if velocity < 0. {
            velocity = 0.;
        }
        let vel = num_to_bytes((velocity * (self.scale as f64)).trunc() as isize);
        let mut msg: Vec<u8> = Vec::with_capacity(vel.len() + self.prefix.len() + 1);
        msg.extend_from_slice(self.prefix.as_slice());
        msg.extend_from_slice(b"SV");
        msg.extend_from_slice(vel.as_slice());
        msg.push(13);
        let resp = self.write(msg.as_slice()).await;
        check_reply(&resp)?;
        Ok(())
    }

    pub async fn set_acceleration(&self, acceleration: f64) -> Result<()> {
        let accel = num_to_bytes((acceleration * (self.scale as f64)).trunc() as isize);
        let mut msg: Vec<u8> = Vec::with_capacity(accel.len() + self.prefix.len() + 1);
        msg.extend_from_slice(self.prefix.as_slice());
        msg.extend_from_slice(b"SA");
        msg.extend_from_slice(accel.as_slice());
        msg.push(13);
        let resp = self.write(msg.as_slice()).await;
        check_reply(&resp)?;
        Ok(())
    }

    pub async fn set_deceleration(&self, deceleration: f64) -> Result<()> {
        let accel = num_to_bytes((deceleration * (self.scale as f64)).trunc() as isize);
        let mut msg: Vec<u8> = Vec::with_capacity(accel.len() + self.prefix.len() + 1);
        msg.extend_from_slice(self.prefix.as_slice());
        msg.extend_from_slice(b"SD");
        msg.extend_from_slice(accel.as_slice());
        msg.push(13);
        let resp = self.write(msg.as_slice()).await;
        check_reply(&resp)?;
        Ok(())
    }

    pub async fn get_status(&self) -> Result<Status> {
        let status_cmd = [2, b'M', self.id + 48, b'G', b'S', 13];
        let res = self.write(status_cmd.as_slice()).await;
        match res[3] {
            48 => Ok(Status::Disabled),
            49 => Ok(Status::Enabling),
            50 => Ok(Status::Faulted),
            51 => Ok(Status::Ready),
            52 => Ok(Status::Moving),
            _ => Err(anyhow!("unknown status".to_string()),
            ),
        }
    }

    pub async fn get_position(&self) -> Result<f64> {
        let get_pos_cmd = [2, b'M', self.id + 48, b'G', b'P', 13];
        let res = self.write(get_pos_cmd.as_slice()).await;
        check_reply(&res)?;
        Ok((ascii_to_int(res.as_slice()) as f64) / (self.scale as f64))
    }

    pub async fn clear_alerts(&self) -> Result<()> {
        let clear_cmd = [2, b'M', self.id + 48, b'C', b'A', 13];
        let resp = self.write(clear_cmd.as_slice()).await;
        check_reply(&resp)?;
        Ok(())
    }

    pub async fn wait_for_move(&self, interval: Duration) -> Result<()> {
        let mut tick_interval = tokio::time::interval(interval);
        tick_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        while self.get_status().await? == Status::Moving {
            tick_interval.tick().await;
        }
        Ok(())
    }
}
