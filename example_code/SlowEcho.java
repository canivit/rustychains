import java.util.Scanner;

public class SlowEcho {
  public static void main(String[] args) throws InterruptedException {
    final Scanner scanner = new Scanner(System.in);
    final String line = scanner.nextLine();
    Thread.sleep(4000);
    System.out.println(line);
  }
}
