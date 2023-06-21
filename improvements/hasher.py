a = input("Enter feature number: ")
b = input("Enter workability level: ")
c = input("Enter timeline level: ")

s = a+" "+b+" "+c

import hashlib
print(str(hex(int(hashlib.sha1(s.encode()).hexdigest(), 16))[2:])[-8:])