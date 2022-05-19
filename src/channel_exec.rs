use constant::{ssh_msg_code, ssh_str};
use error::SshResult;
use packet::Data;
use crate::channel::Channel;
use crate::util;



pub struct ChannelExec(pub(crate) Channel);

impl ChannelExec {

    fn exec_command(&self, command: &str) -> SshResult<()> {
        println!("exec_command");
        let mut data = Data::new();
        data.put_u8(ssh_msg_code::SSH_MSG_CHANNEL_REQUEST)
            .put_u32(self.0.server_channel)
            .put_str(ssh_str::EXEC)
            .put_u8(true as u8)
            .put_str(command);
        let mut client = util::client()?;
        client.write(data)
    }

    fn get_data(&mut self, v: &mut Vec<u8>) -> SshResult<()> {
        let mut client = util::client()?;
        let results = client.read()?;
        util::unlock(client);
        for mut result in results {
            if result.is_empty() { continue }
            let message_code = result.get_u8();
            match message_code {
                ssh_msg_code::SSH_MSG_CHANNEL_DATA => {
                    let cc = result.get_u32();
                    if cc == self.0.client_channel {
                        v.append(&mut result.get_u8s());
                    }
                }
                ssh_msg_code::SSH_MSG_CHANNEL_CLOSE => {
                    let cc = result.get_u32();
                    if cc == self.0.client_channel {
                        self.0.remote_close = true;
                        self.0.close()?;
                    }
                }
                _ => self.0.other(message_code, result)?
            }
        }
        Ok(())
    }

    pub fn send_command(mut self, command: &str) -> SshResult<Vec<u8>> {
        self.exec_command(command)?;
        let mut r = vec![];
        loop {
            self.get_data(&mut r)?;
            if self.0.remote_close
                && self.0.local_close
            {
                break
            }
        }
        Ok(r)
    }
}
