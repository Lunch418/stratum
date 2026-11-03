# Опыт Wine: замена блока гиперссылки в .vdr с правкой размеров (см. docs/formats/vdr.md)
import struct,sys
def patch(src,dst,fields):
    """fields: list of (id, bytes) written in the given order"""
    d=bytearray(open(src,'rb').read())
    i=d.find(b'\xfe\x03\x1d\x00\x00\x00'); assert i>0
    body=b''.join(struct.pack('<HH',fid,len(v))+v if fid!=4 else struct.pack('<HH',4,v) for fid,v in fields)
    blk=bytes([1,0,5,0,2,0,1])+struct.pack('<HH',0x0a,len(body))+body
    new=b'\xfe\x03'+struct.pack('<I',6+len(blk))+blk
    old_len=29
    delta=len(new)-old_len
    end=i+old_len
    rec=[p for p in range(i-150,i) if d[p:p+2]==b'\x03\x00' and p+struct.unpack_from('<I',d,p+2)[0]==end]
    assert len(rec)==1
    d[i:i+old_len]=new
    struct.pack_into('<I',d,rec[0]+2,struct.unpack_from('<I',d,rec[0]+2)[0]+delta)
    for off in (4,16,176): 
        v=struct.unpack_from('<I',d,off)[0]; struct.pack_into('<I',d,off,v+delta)
    open(dst,'wb').write(d)
