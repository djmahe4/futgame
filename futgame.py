import random

form1=4,4,2
form2=4,4,2

onehasball=True
goal=[]
miss=[]
save=[]
count=0
game = True

with open('desc.txt') as file:
    comments=file.readlines()
    for i in range(16):
        save.append(comments[i])
    for i in range(17, 24):
        miss.append(comments[i])
    for i in range(25, 49):
        goal.append(comments[i])

toss=input("Heads or Tails(H/T):")
ts=toss.lower()

choices = ["g","1","2","3","4","5","6","7","8","9","0"]
options = choices
score1 = 0
score2 = 0
counter = 90
prev=''
poss1=0
poss2=0

def check_game():
    if count==45:
        print("Half time")
        printsc()
    elif count>90:
        print("Full time")
        printsc()
        game =False
        global game
        return True
def printsc():
    print("Team1:", score1)
    print("Team2:", score2)
    print("Kickoff!!")
    if onehasball==True:
        print("has possession")

if ts=="t" or ts=="h":
    fall = random.choice(['h','t'])
    if ts==fall:
        onehasball = True
        printsc()
    else:
        onehasball = False
        printsc()

def selection(one):
    options=[]
    for i in range(11):
        if one=="g":
            options=["9","0","g","1","2"]
        elif one=="1":
            options = ["0", "g", "1", "2", "3"]
        elif one == "2":
            options = ["g", "1", "2", "3", "4"]
        elif one == "3":
            options = ["1", "2", "3", "4", "5"]
        elif one == "4":
            options = ["2", "3", "4", "5", "6"]
        elif one == "5":
            options = ["3", "4", "5", "6", "7"]
        elif one == "6":
            options = ["4", "5", "6", "7", "8"]
        elif one == "7":
            options = ["5", "6", "7", "8", "9"]
        elif one == "8":
            options = ["6", "7", "8", "9", "0"]
        elif one=="9":
            options = ["7", "8", "9", "0", "g"]
        elif one=="0":
            options = ["8", "9", "0", "g", "1"]
    return options

def predict(x):
    y = random.choice(x)
    return y

while game==True:
    for i in range(counter):
        count += 1
        if check_game() == True:
            break
        elif onehasball == True:
            print("Options:", options)
            one = str(input("Enter choice:"))
            x = selection(one)
            two = predict(x)
            print("guess:", two)
            print(count, "'")
            if two == one:
                onehasball = False
                print("Possession lost!")
                poss1 = 0
                break
            elif prev == one and two != one:
                nerves = random.choice([0, 1, 2, 3])
                if nerves == 0:
                    print(random.choice(goal))
                    score1 += 1
                    onehasball = False
                    printsc()
                    poss1 = 0
                    break
                elif nerves == 1:
                    print("possession lost!")
                    onehasball = False
                elif nerves == 2:
                    print(random.choice(save))
                    onehasball = True
                elif nerves == 3:
                    print(random.choice(miss))
                    print("possession lost!")
                    onehasball = False

            else:
                if poss1 >= 10:
                    score1 += 1
                    onehasball = False
                    printsc()
                    poss1 = 0
                    break
            prev = one
            options = x
            poss1 += 1
        elif onehasball == False:
            two = random.choice(options)
            print("Options:", options)
            one = str(input("Enter guess:"))
            x = selection(two)
            print("attempt:", two)
            print(count, "'")
            if two == one:
                onehasball = True
                print("Possession gained!")
                poss2 = 0
            elif prev == two and two != one:
                nerves = random.choice([0, 1, 2, 3])
                if nerves == 0:
                    print(random.choice(goal))
                    score2 += 1
                    onehasball = True
                    printsc()
                    poss2 = 0
                elif nerves == 1:
                    print("possession gained!")
                    onehasball = True
                elif nerves == 2:
                    print(random.choice(save))
                    onehasball = False
                elif nerves == 3:
                    print(random.choice(miss))
                    print("possession gained!")
                    onehasball = True
            else:
                if poss2 >= 10:
                    score2 += 1
                    twohasball = False
                    onehasball = True
                    printsc()
                    poss2 = 0
            prev = two
            options = x
            poss2 += 1