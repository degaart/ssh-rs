use error::SshResult;
use packet::Data;
use constant::ssh_msg_code;
use crate::channel::Channel;
use crate::util;


pub struct ChannelShell(pub(crate) Channel);

impl ChannelShell {
    pub fn read(&mut self) -> SshResult<Vec<u8>> {
        let mut buf = vec![];
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
                        let mut vec = result.get_u8s();
                        buf.append(&mut vec);
                    }
                }
                _ => self.0.other(message_code, result)?
            }
        }
        Ok(buf)
    }

    pub fn write(&self, buf: &[u8]) -> SshResult<()> {
        let mut data = Data::new();
        data.put_u8(ssh_msg_code::SSH_MSG_CHANNEL_DATA)
            .put_u32(self.0.server_channel)
            .put_u8s(buf);
        let mut client = util::client()?;
        client.write(data)
    }

    pub fn close(mut self) -> SshResult<()> {
        self.0.close()
    }

}
